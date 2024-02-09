use std::fmt::Display;
use std::{alloc::Layout, io::Write};
use async_rdma::{LocalMr, LocalMrWriteAccess, Rdma, RdmaBuilder, RemoteMr, MTU};
use log::{error, info};
use tonic::{transport::Server, Request, Response, Status};
use crate::initiator::listener::listener::listener_server::{Listener, ListenerServer};
use crate::initiator::listener::listener::{
    SendReply, SendRequest, Operation, Mtu
};
use crate::server::connection_manager::connection_manager::{
    ConnectRequest,
    connection_client::ConnectionClient
};

#[derive(Debug, Default)]
pub struct Initiator{
    address: String,
}

impl Initiator {
    pub fn new(address: String) -> Initiator {
        Initiator{
            address,
        }
    }
    pub async fn run(&mut self) -> anyhow::Result<()> {
        info!("starting initiator at {}", self.address);
        let addr = self.address.parse().unwrap();
        let listener = Initiator::default();
        Server::builder()
            .add_service(ListenerServer::new(listener))
            .serve(addr)
            .await?;
        Ok(())
    }
}

pub async fn initiate(request: SendRequest, uuid: String) -> anyhow::Result<()> {
    let init_address = format!("http://{}",request.address.clone());
    info!("connecting to server at {}", init_address);
    let mtu: Mtu = Mtu::try_from(request.mtu).unwrap();
    let (mtu, mtu_2) = match mtu {
        Mtu::Mtu512 => (512, MTU::MTU512),
        Mtu::Mtu1024 => (1024, MTU::MTU1024),
        Mtu::Mtu2048 => (2048, MTU::MTU2048),
        Mtu::Mtu4096 => (4096, MTU::MTU4096)
    };
    let mut init_client = ConnectionClient::connect(init_address).await?;
    let connect_request = tonic::Request::new(ConnectRequest{
        uuid: uuid.to_string(),
        iterations: request.iterations,
        message_size: request.message_size,
        volume: request.volume,
        mtu
    });
    let response = init_client.init(connect_request).await?;
    let port = response.get_ref().port;
    let address = request.address.split(":").next().unwrap();
    let server_address = format!("{}:{}", address, port);
    info!("connecting to server at {} for request {}", server_address, uuid);
    let op = Operation::try_from(request.op).unwrap();
    let mut stats_map = StatsMap::new();
    info!("sending {} iterations with size {} for request {}", request.iterations, request.message_size, uuid);
    let mut rdma = RdmaBuilder::default().
    set_max_message_length(request.message_size as usize).
    set_mtu(mtu_2).
    connect(server_address.clone()).await.unwrap();
    for it in 0..request.iterations{
        let mut message_count = 0;
        let mut total_message_size: u64 = 0;
        let sends = request.volume/request.message_size;
        
        rdma = rdma.new_connect(server_address.clone()).await.unwrap();
        info!("allocating {} bytes for request {}", request.message_size as usize, uuid.clone());

        let layout = Layout::from_size_align(request.message_size as usize, 1).unwrap();
        let mut lmr = match rdma.alloc_local_mr(layout){
            Ok(lmr) => lmr,
            Err(e) => {
                panic!("alloc_local_mr error: {}", e);
            }
        };
        info!("allocation done, writing to memory");
        let buf = vec![1_u8; request.message_size as usize];
        let _num = lmr.as_mut_slice().write(buf.as_slice()).unwrap();

        
        let mut rmr = match op{
            Operation::Write => {
                let rmr = rdma.request_remote_mr(layout).await?;
                Some(rmr)
            },
            Operation::WriteWithImm => {
                let rmr = rdma.request_remote_mr(layout).await?;
                Some(rmr)
            },
            _ => {
                None
            },
        };

        let start = tokio::time::Instant::now();
        info!("writing done, sending message");
        for _ in 0..sends{
            let res = match op{
                Operation::Send => {
                    send(&rdma, &lmr).await
                },
                Operation::SendWithImm => {
                    send_with_imm(&rdma, &lmr).await
                },
                Operation::Write => {
                    write(&rdma, &lmr, rmr.as_mut().unwrap()).await
                },
                Operation::WriteWithImm => {
                    write_with_imm(&rdma, &lmr, rmr.as_mut().unwrap()).await
                },
            };
            match res {
                Ok(_) => {
                    message_count += 1;
                    total_message_size += request.message_size;
                },
                Err(e) => error!("operation error: {}", e),
            }
        }
        let stats = Stats::new(total_message_size, message_count, start, uuid.clone());
        stats_map.push(it, stats);
    }
    info!("\n{}", stats_map);
    Ok(())
}

pub async fn send(rdma: &Rdma, lmr: &LocalMr) -> anyhow::Result<()> {
    rdma.send(&lmr).await?;
    Ok(())
}

pub async fn send_with_imm(rdma: &Rdma, lmr: &LocalMr) -> anyhow::Result<()> {
    rdma.send_with_imm(&lmr, 1_u32).await?;
    Ok(())
}

pub async fn write(rdma: &Rdma, lmr: &LocalMr, rmr: &mut RemoteMr) -> anyhow::Result<()> {
    rdma.write(lmr, rmr).await?;
    Ok(())
}

async fn write_with_imm(rdma: &Rdma, lmr: &LocalMr, rmr: &mut RemoteMr) -> anyhow::Result<()> {
    rdma.write_with_imm(lmr, rmr, 1_u32).await?;
    Ok(())
}

#[tonic::async_trait]
impl Listener for Initiator {
    async fn send(
        &self,
        request: Request<SendRequest>,
    ) -> Result<Response<SendReply>, Status> {
        let request = request.into_inner();
        let uuid = uuid::Uuid::new_v4();
        tokio::task::spawn(async move{
            if let Err(e) = initiate(request, uuid.to_string()).await{
                error!("initiate error: {:?}", e);
                return Err(Status::internal(e.to_string()));
            }
            Ok(())
        });

        let reply = SendReply{
            uuid: uuid.to_string(),
        };

        Ok(Response::new(reply))
    }
}

#[derive(Debug, Default)]
pub struct StatsMap(Vec<(u32, Stats)>);

impl StatsMap{
    pub fn new() -> StatsMap{
        StatsMap(Vec::new())
    }
    pub fn push(&mut self, it: u32, stats: Stats){
        self.0.push((it, stats));
    }
}

impl Display for StatsMap{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        for (it, stats) in self.0.iter(){
            s.push_str(&format!("iteration: {}\n{}\n", it, stats));
            
        }
        write!(f, "{}",s)
    }
}

#[derive(Debug, Default)]
pub struct Stats{
    pub messages: u32,
    pub total_message_size: u64,
    pub total_time: String,
    pub rate: String,
    pub uuid: String,
}

impl Stats{
    pub fn new(total_message_size: u64, messages: u32, start: tokio::time::Instant, uuid: String) -> Stats{
        let bytes = total_message_size;
        let bits = (bytes*8) as f64;
        let seconds = start.elapsed().as_secs_f64();
        let milliseconds = start.elapsed().as_millis();

        let bps = if seconds > 0.0 {
            bits/seconds
        } else {
            let factor = milliseconds as f64/1000.0;
            let bits = bits + factor;
            bits
        };

        let total_time: String = if milliseconds > 1000 {
            format!("{:.2} s", seconds)
        } else {
            format!("{} ms", milliseconds)
        };

        let rate = if bps > 1_000_000_000.0 {
            let rate = bps/1_000_000_000.0;
            format!("{:.2} Gbps", rate)
        } else if bps > 1_000_000.0 {
            let rate = bps/1_000_000.0;
            format!("{:.2} Mbps", rate)
        } else if bps > 1_000.0 {
            let rate = bps/1_000.0;
            format!("{:.2} Kbps", rate)
        } else {
            let rate = bps;
            format!("{:.2} bps", rate)
        };

        Stats{
            messages,
            total_message_size: bytes,
            total_time,
            rate,
            uuid,
        }
    }
}

impl Display for Stats{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\tuid: {}\n\tmessages: {}\n\ttotal message size: {}\n\ttotal time: {}\n\trate: {}",self.uuid, self.messages, self.total_message_size, self.total_time, self.rate)
    }
}
