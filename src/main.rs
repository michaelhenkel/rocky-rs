use std::fmt::Display;
use clap::Parser;
use rocky_rs::connection_manager::connection_manager::server_connection_client::ServerConnectionClient;
use rocky_rs::connection_manager::connection_manager::{Mode, Mtu, Operation};
use rocky_rs::connection_manager::connection_manager::{
    server_connection_server::{
        ServerConnection, ServerConnectionServer,
    },
    initiator_connection_server::{
        InitiatorConnection, InitiatorConnectionServer,
    },
    InitiatorReply, Request,
    ServerReply,
};
use serde::Deserialize;
use tonic::{transport::Server as GrpcServer,Request as GrpcRequest, Status};
use log::{error, info};
use tokio::process::Command;
use regex::Regex;

#[derive(Parser, Debug)]
struct Args{
    #[arg(short, long)]
    address: String,
    #[arg(short, long)]
    port: u16,
    #[arg(short, long)]
    device: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()>{
    env_logger::init();
    let args = Args::parse();
    let rocky = Rocky::new(args.address, args.port, args.device);
    rocky.run().await
}

#[derive(Debug, Default, Clone)]
pub struct Rocky{
    address: String,
    port: u16,
    device: Option<String>,
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
        tokio::spawn(async move{
            if let Err(e) = listen(request, device, port).await{
                error!("listen error: {:?}", e);
                return Err(Status::internal(e.to_string()));
            }
            info!("server done with request {}", uuid.unwrap());
            Ok(())
        });
        Ok(tonic::Response::new(reply))
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
        tokio::task::spawn(async move{
            if let Err(e) = initiate(request, device).await{
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


impl Rocky{
    pub fn new(address: String, port: u16, device: Option<String>) -> Rocky {
        Rocky{
            address,
            port,
            device,
        }
    }
    pub async fn run(self) -> anyhow::Result<()> {
        let address = format!("{}:{}", self.address, self.port);
        info!("starting grpc server at {}", address);
        let addr = address.parse().unwrap();
        let mut rocky = Rocky::default();
        rocky.device = self.device.clone();
        rocky.address = self.address.clone();
        rocky.port = self.port;
        GrpcServer::builder()
        .add_service(ServerConnectionServer::new(rocky.clone()))
        .add_service(InitiatorConnectionServer::new(rocky))
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

pub async fn listen(mut request: Request, device_name: Option<String>, port: u16) -> anyhow::Result<()> {
    info!("listening on port {}", port);
    request.server_port = port as u32;
    let mut cmd = request_to_cmd(request.clone(), device_name, "server");
    info!("server cmd: {:?}", cmd);
    cmd.output().await?;
    let output = std::fs::read_to_string(format!("/tmp/{}-server.json", request.uuid.unwrap()))?;
    let quoted_string = quote_strings(&output);
    info!("quoted_string: {}", quoted_string);
    let results: Report = serde_json::from_str(&quoted_string)?;
    info!("results: {:#?}", results);
    Ok(())
}

pub async fn initiate(mut request: Request, device: Option<String>) -> anyhow::Result<()> {
    let server_address = format!("http://{}:{}", request.server_address.clone(), request.server_port);
    let mut server_client = ServerConnectionClient::connect(server_address).await?;
    let grpc_request = GrpcRequest::new(request.clone());
    let server_reply = server_client.server(grpc_request).await?.into_inner();
    let port = server_reply.port;
    request.server_port = port as u32;
    let mut cmd = request_to_cmd(request.clone(), device, "initiator");
    cmd.arg(request.server_address);
    info!("initiator cmd: {:?}", cmd);
    cmd.output().await?;
    let output = std::fs::read_to_string(format!("/tmp/{}-initiator.json", request.uuid.unwrap()))?;
    let quoted_string = quote_strings(&output);
    info!("quoted_string: {}", quoted_string);
    let results: Report = serde_json::from_str(&quoted_string)?;
    info!("results: {:#?}", results);
    Ok(())
}

#[derive(Debug, Deserialize)]
struct Report{
    test_info: TestInfo,
    results: BwResults,
}

#[derive(Debug, Deserialize)]
struct TestInfo{
    test: String,
    #[serde(rename(deserialize = "Dual_port"))]
    dual_port: String,
    #[serde(rename(deserialize = "Device"))]
    device: String,
    #[serde(rename(deserialize = "Number_of_qps"))]
    number_of_qps: u32,
    #[serde(rename(deserialize = "Transport_type"))]
    transport_type: String,
    #[serde(rename(deserialize = "Connection_type"))]
    connection_type: String,
    #[serde(rename(deserialize = "Using_SRQ"))]
    using_srq: String,
    #[serde(rename(deserialize = "PCIe_relax_order"))]
    pci_relax_order: String,
    #[serde(rename(deserialize = "ibv_wr_API"))]
    ibv_wr_api: String,
    #[serde(rename(deserialize = "TX_depth"))]
    tx_depth: Option<u32>,
    #[serde(rename(deserialize = "RX_depth"))]
    rx_depth: Option<u32>,
    #[serde(rename(deserialize = "CQ_Moderation"))]
    cq_moderation: u32,
    #[serde(rename(deserialize = "Mtu"))]
    mtu: u32,
    #[serde(rename(deserialize = "Link_type"))]
    link_type: String,
    #[serde(rename(deserialize = "GID_index"))]
    gid_index: u32,
    #[serde(rename(deserialize = "Max_inline_data"))]
    max_inline_data: u32,
    #[serde(rename(deserialize = "rdma_cm_QPs"))]
    rdma_cm_qps: String,
    #[serde(rename(deserialize = "Data_ex_method"))]
    data_ex_method: String,
}


#[derive(Debug, Deserialize)]
struct BwResults{
    #[serde(rename(deserialize = "MsgSize"))]
    msg_size: u32,
    #[serde(rename(deserialize = "n_iterations"))]
    n_iterations: u32,
    #[serde(rename(deserialize = "BW_peak"))]
    bw_peak: f32,
    #[serde(rename(deserialize = "BW_average"))]
    bw_average: f32,
    #[serde(rename(deserialize = "MsgRate"))]
    msg_rate: f32,
}

impl Display for Report{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Test Info: {}\nResults: {}", self.test_info, self.results)
    }
}

impl Display for TestInfo{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Test Info: {}", self)
    }
}

impl Display for BwResults{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Results: {}", self)
    }
}

fn quote_strings(input: &str) -> String {
    let re = Regex::new(r"\b([a-zA-Z]\w*)\b").unwrap();
    let result = re.replace_all(input, "\"$1\"");
    let re = Regex::new(r#""""#).unwrap(); 
    let result = re.replace_all(&result, "\"");
    let re = Regex::new(r",\s*([\]}])").unwrap();
    let result = re.replace_all(&result, "$1");
    result.into_owned()
}
