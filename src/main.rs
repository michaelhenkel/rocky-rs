use clap::Parser;
use rocky_rs::stats_manager::counter_monitor::counter_monitor::{CounterMonitor, CounterMonitorClient};
use tokio_stream::{wrappers::ReceiverStream, Stream};
use rocky_rs::connection_manager::connection_manager::server_connection_client::ServerConnectionClient;
use rocky_rs::connection_manager::connection_manager::{Mode, Mtu, Operation};
use rocky_rs::connection_manager::connection_manager::{
    server_connection_server::{
        ServerConnection, ServerConnectionServer,
    },
    initiator_connection_server::{
        InitiatorConnection, InitiatorConnectionServer,
    },
    stats_manager_server::{
        StatsManager as GrpcStatsManager, StatsManagerServer,
    },
    monitor_server::{
        Monitor, MonitorServer,
    },
    InitiatorReply, Request,
    ServerReply, Report, ReportReply, ReportList, ReportRequest,
    CounterFilter, InterfaceCounter
};
use tonic::{transport::Server as GrpcServer,Request as GrpcRequest, Status, Response};
use log::{error, info};
use tokio::process::Command;
use std::pin::Pin;
use std::process::Stdio;
use port_scanner;
use rocky_rs::stats_manager::stats_manager::{Client, StatsManager};

type MonitorResult<T> = anyhow::Result<Response<T>, Status>;
type ResponseStream = Pin<Box<dyn Stream<Item = Result<InterfaceCounter, Status>> + Send>>;



#[derive(Parser, Debug)]
struct Args{
    #[arg(short, long)]
    address: String,
    #[arg(short, long)]
    port: u16,
    #[arg(short, long)]
    device: Option<String>,
    #[arg(short, long)]
    frequency: Option<u64>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()>{
    env_logger::init();
    let args = Args::parse();
    let stats_manager = StatsManager::new();
    let counter_monitor = CounterMonitor::new(args.frequency.unwrap_or(2000));
    let counter_monitor_client = counter_monitor.client();
    let rocky = Rocky::new(args.address, args.port, args.device, stats_manager.client(), counter_monitor_client);
    let _res = tokio::join!(rocky.run(), stats_manager.run(), counter_monitor.run());
    Ok(())
}

#[derive(Clone)]
pub struct Rocky{
    address: String,
    port: u16,
    device: Option<String>,
    client: Client,
    monitor_client: CounterMonitorClient,
}

#[tonic::async_trait]
impl Monitor for Rocky {
    type MonitorStreamStream = ResponseStream;
    async fn monitor_stream(
        &self,
        req: GrpcRequest<CounterFilter>,
    ) -> MonitorResult<ResponseStream> {
        let req = req.into_inner();
        info!("monitor stream request: {:?}", req.clone());
        let (tx, rx) = tokio::sync::mpsc::channel(128);
        let (monitor_tx, mut monitor_rx) = tokio::sync::mpsc::channel(120);
        let id = uuid::Uuid::new_v4().to_string();
        info!("registering monitor client with id: {}", id.clone());
        self.monitor_client.register(id.clone(), monitor_tx, req).await.unwrap();
        let monitor_client = self.monitor_client.clone();
        tokio::spawn(async move{
            while let Some(counter) = monitor_rx.recv().await{
                match tx.send(Result::<_, Status>::Ok(counter)).await {
                    Ok(_) => {},
                    Err(e) => {
                        error!("monitor send error: {:?}", e);
                        break;
                    }
                }
            }
            info!("monitor done");
            monitor_client.unregister(id).await.unwrap();
        });
        let output_stream = ReceiverStream::new(rx);
        Ok(Response::new(
            Box::pin(output_stream) as ResponseStream
        ))
    }

}

#[tonic::async_trait]
impl ServerConnection for Rocky {
    async fn server(
        &self,
        request: tonic::Request<Request>,
    ) -> Result<tonic::Response<ServerReply>, tonic::Status> {
        let port = portpicker::pick_unused_port().unwrap();
        let reply = ServerReply{
            port: port as u32,
        };
        let request = request.into_inner();
        let device = self.device.clone();
        let uuid = request.uuid.clone();
        let (tx, rx) = tokio::sync::oneshot::channel();
        let client = self.client.clone();
        tokio::spawn(async move{
            if let Err(e) = listen(request, device, port, tx, client).await{
                error!("listen error: {:?}", e);
                return Err(Status::internal(e.to_string()));
            }
            info!("server done with request {}", uuid.unwrap());
            Ok(())
        });
        match rx.await{
            Ok(_) => {
                Ok(tonic::Response::new(reply))
            },
            Err(e) => {
                error!("rx error: {:?}", e);
                return Err(Status::internal(e.to_string()));
            }
        }
    }
}

#[tonic::async_trait]
impl InitiatorConnection for Rocky {
    async fn initiator(
        &self,
        request: tonic::Request<Request>,
    ) -> Result<tonic::Response<InitiatorReply>, tonic::Status> {

        let mut request = request.into_inner();
        let uuid = uuid::Uuid::new_v4();
        let device = self.device.clone();
        request.uuid = Some(uuid.to_string());
        let client = self.client.clone();
        tokio::task::spawn(async move{
            if let Err(e) = initiate(request, device, client).await{
                error!("initiate error: {:?}", e);
                return Err(Status::internal(e.to_string()));
            }
            info!("initiator done with request {}", uuid.to_string());
            Ok(())
        });
        let reply = InitiatorReply{
            uuid: uuid.to_string(),
        };
        Ok(tonic::Response::new(reply))
    }
}

#[tonic::async_trait]
impl GrpcStatsManager for Rocky {
    async fn get_report(
        &self,
        request: tonic::Request<ReportRequest>,
    ) -> Result<tonic::Response<ReportReply>, tonic::Status> {
        let request = request.into_inner();
        let suffix = request.suffix;
        let uuid = request.uuid;
        let report = match self.client.get(uuid.clone(), suffix.clone()).await{
            Ok(report) => report,
            Err(e) => {
                error!("get error: {:?}", e);
                return Err(Status::internal(e.to_string()));
            },
        };
        let report_reply = if let Some(report) = report{
            let report: Report = report.into();
            Some(report)
        } else {
            None
        };
        
        let report_reply = ReportReply{
            report: report_reply,
        };
        Ok(tonic::Response::new(report_reply))
    }
    async fn list_report(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<ReportList>, tonic::Status> {

        let reports = match self.client.list().await{
            Ok(reports) => reports,
            Err(e) => {
                error!("list error: {:?}", e);
                return Err(Status::internal(e.to_string()));
            },
        };
        let mut report_list = ReportList::default();
        for ((uuid, suffix), report) in reports{
            let key = format!("{}__{}", uuid, suffix);
            let report: Report = report.into(); 
            report_list.reports.insert(key, report);
        }
        Ok(tonic::Response::new(report_list))
    }
    async fn delete_report(
        &self,
        request: tonic::Request<ReportRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        let request = request.into_inner();
        let uuid = request.uuid;
        let suffix = request.suffix;
        if let Err(e) = self.client.remove(uuid, suffix).await{
            error!("delete error: {:?}", e);
            return Err(Status::internal(e.to_string()));
        }
        Ok(tonic::Response::new(()))
    }
}


impl Rocky{
    pub fn new(address: String, port: u16, device: Option<String>, client: Client, monitor_client: CounterMonitorClient) -> Rocky {
        Rocky{
            address,
            port,
            device,
            client,
            monitor_client,
        }
    }
    pub async fn run(self) -> anyhow::Result<()> {
        let address = format!("{}:{}", self.address, self.port);
        info!("starting grpc server at {}", address);
        let addr = address.parse().unwrap();
        GrpcServer::builder()
        .add_service(ServerConnectionServer::new(self.clone()))
        .add_service(InitiatorConnectionServer::new(self.clone()))
        .add_service(StatsManagerServer::new(self.clone()))
        .add_service(MonitorServer::new(self))
        .serve(addr)
        .await?;
        Ok(())
    }
}

pub fn request_to_cmd(request: Request, device_name: Option<String>, suffix: &str) -> Command{
    let mode = match request.mode(){
        Mode::Bw => {
            "bw"
        },
        Mode::Lat => {
            "lat"
        },
    };
    let cmd = match request.operation(){
        Operation::Atomic => { format!("/usr/bin/ib_atomic_{}", mode) },
        Operation::Write => { format!("/usr/bin/ib_write_{}", mode) },
        Operation::Send => { format!("/usr/bin/ib_send_{}", mode) },
        Operation::Read => { format!("/usr/bin/ib_read_{}", mode) },
    };
    let mut cmd = Command::new(cmd);

    if let Some(device) = device_name{
        cmd.arg("-d").arg(device);
    }

    if request.mtu.is_some(){
        let mtu = match request.mtu(){
            Mtu::Mtu512 => { 512 },
            Mtu::Mtu1024 => { 1024 },
            Mtu::Mtu2048 => { 2048 },
            Mtu::Mtu4096 => { 4096 },
        };
        cmd.arg("-m").arg(mtu.to_string());
    }

    if request.iterations.is_some(){
        cmd.arg("-n").arg(request.iterations().to_string());
    }

    cmd.arg("-p").arg(request.server_port.to_string());
    if let Some(msg_size) = request.message_size{
        cmd.arg("-s").arg(msg_size.to_string());
    }

    cmd.arg("--out_json").arg("--out_json_file").arg(format!("/tmp/{}-{}.json", request.uuid.unwrap(), suffix));
    if request.cm{
        cmd.arg("-R");
    }

    cmd
}

pub async fn listen(mut request: Request, device_name: Option<String>, port: u16, tx: tokio::sync::oneshot::Sender<bool>, client: Client) -> anyhow::Result<()> {
    info!("listening on port {}", port);
    request.server_port = port as u32;
    let mut cmd = request_to_cmd(request.clone(), device_name, "server");
    let mut child = cmd.
        stdout(Stdio::piped()).
        stderr(Stdio::piped()).
        kill_on_drop(true).
        spawn()?;

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        if !port_scanner::local_port_available(port){
            break;
        }
    }
    tx.send(true).unwrap();

    child.wait().await?;
    client.add(request.clone().uuid().to_string(), "server".to_string()).await?;
    Ok(())
}

pub async fn initiate(mut request: Request, device: Option<String>, client: Client) -> anyhow::Result<()> {
    let server_address = format!("http://{}:{}", request.server_address.clone(), request.server_port);
    let mut server_client = ServerConnectionClient::connect(server_address.clone()).await?;
    info!("initiating connection to server at {}", server_address);
    let grpc_request = GrpcRequest::new(request.clone());
    let server_reply = server_client.server(grpc_request).await?.into_inner();
    let port = server_reply.port;
    request.server_port = port as u32;
    let mut cmd = request_to_cmd(request.clone(), device, "initiator");
    cmd.arg(request.clone().server_address);
    let _out = cmd.output().await?;
    //let report = stats_manager::Report::new(request.clone().uuid(), "initiator");
    client.add(request.clone().uuid().to_string(), "initiator".to_string()).await?;
    Ok(())
}

