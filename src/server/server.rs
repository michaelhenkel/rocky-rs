use async_rdma::{LocalMrReadAccess, MrAccess, Rdma, RdmaBuilder, MTU};
use crate::server::connection_manager::connection_manager::{
    connection_server::{Connection, ConnectionServer},
    ConnectReply, ConnectRequest,
};
use tonic::transport::Server as GrpcServer;
use log::{error, info};
use portpicker;
use crate::initiator::initiator::{Stats,StatsMap};

use super::connection_manager::connection_manager::Operation;

pub struct Server{
    address: String,
    port: u16,
}

#[tonic::async_trait]
impl Connection for Server {
    async fn init(
        &self,
        request: tonic::Request<ConnectRequest>,
    ) -> Result<tonic::Response<ConnectReply>, tonic::Status> {
        info!("init request {}", request.get_ref().uuid);
        let uuid = request.get_ref().uuid.clone();
        let port = portpicker::pick_unused_port().unwrap();
        let address = self.address.clone();
        let iterations = request.get_ref().iterations;
        let message_size = request.get_ref().message_size;
        let mtu = request.get_ref().mtu;
        let volume = request.get_ref().volume;
        let op = request.get_ref().operation();
        info!("spawning listener at {}:{}", address, port);
        tokio::spawn(async move{
            listener(address, port, iterations, message_size, volume, mtu, uuid, op).await
        });
        let reply = ConnectReply{
            port: port as u32,
        };
        Ok(tonic::Response::new(reply))
    }
}

impl Server {
    pub fn new(address: String, port: u16) -> Server {
        Server{
            address,
            port,
        }
    }
    pub async fn run(self) -> anyhow::Result<()> {
        let address = format!("{}:{}", self.address, self.port);
        info!("starting grpc server at {}", address);
        let addr = address.parse().unwrap();
        GrpcServer::builder()
        .add_service(ConnectionServer::new(self))
        .serve(addr)
        .await?;
        Ok(())
    }


}

async fn listener(address: String, port: u16, iterations: u32, message_size: u64, volume: u64, mtu: u32, uuid: String, op: Operation) -> anyhow::Result<()> {
    let address = format!("{}:{}", address, port);
    let mtu = match mtu {
        512 => MTU::MTU512,
        1024 => MTU::MTU1024,
        2048 => MTU::MTU2048,
        4096 => MTU::MTU4096,
        _ => MTU::MTU1024,
    };
 
    info!("listening for rdma at {} for request {}", address, uuid);
    info!("expecting {} iterations with size {}", iterations, message_size);
    let mut stats_map = StatsMap::new();
    let mut rdma = RdmaBuilder::default().
    set_max_message_length(message_size as usize).
    set_mtu(mtu).
    listen(address.clone()).await?;
    let receives = volume/message_size;
    for it in 0..iterations{
        let mut total_message_size: u64 = 0;
        let mut message_count = 0;
        rdma = rdma.listen().await?;
        let start = tokio::time::Instant::now();
        for _ in 0..receives{
            let res = match op{
                Operation::Send => {
                    receive(&rdma).await
                },
                Operation::SendWithImm => {
                    receive_with_imm(&rdma).await
                },
                Operation::Write => {
                    receive_local_mr(&rdma).await
                },
                Operation::WriteWithImm => {
                    receive_local_mr_with_imm(&rdma).await
                },
            };
            match res {
                Ok(len) => {
                    total_message_size += len as u64;
                    message_count += 1;
                },
                Err(e) => {
                    error!("receive error: {}", e);
                }
            }
        }
        info!("done with iteration {} for request {}", it, uuid);
        let stats = Stats::new(total_message_size, message_count, start, uuid.clone());
        stats_map.push(it, stats);
    }
    info!("\n{}", stats_map);
    Ok(())
}

pub async fn receive(rdma: &Rdma) -> anyhow::Result<usize> {
    let lmr = rdma.receive().await?;
    let _data = *lmr.as_slice();
    Ok(lmr.length())
    
}

pub async fn receive_with_imm(rdma: &Rdma) -> anyhow::Result<usize> {
    let (lmr, _imm) = rdma.receive_with_imm().await?;
    let _data = *lmr.as_slice();
    Ok(lmr.length())
}

pub async fn receive_local_mr(rdma: &Rdma) -> anyhow::Result<usize> {
    let lmr = rdma.receive_local_mr().await?;
    let _data = *lmr.as_slice();
    Ok(lmr.length())
}

pub async fn receive_local_mr_with_imm(rdma: &Rdma) -> anyhow::Result<usize> {
    let _imm = rdma.receive_write_imm().await?;
    let lmr = rdma.receive_local_mr().await?;
    let _data = *lmr.as_slice();
    Ok(lmr.length())
}