use async_rdma::{LocalMrReadAccess, Rdma, RdmaBuilder, MTU};
use crate::server::connection_manager::connection_manager::{
    connection_server::{Connection, ConnectionServer},
    ConnectReply, ConnectRequest,
};
use tonic::transport::Server as GrpcServer;
use log::{error, info};
use portpicker;

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
        info!("init request from {}", request.get_ref().id);
        let port = portpicker::pick_unused_port().unwrap();
        let address = self.address.clone();
        let messages = request.get_ref().messages;
        let message_size = request.get_ref().message_size;
        let mtu = request.get_ref().mtu;
        info!("spawning listener at {}:{}", address, port);
        tokio::spawn(async move{
            listener(address, port, messages, message_size, mtu).await
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
        let res = tokio::join!(
            self.grpc_server(),
        );
        match res {
            (Ok(res),) => Ok(res),
            (Err(e),) => Err(e),
        }
    }
    async fn grpc_server(self) -> anyhow::Result<()> {
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

async fn listener(address: String, port: u16, messages: u32, message_size: u32, mtu: u32) -> anyhow::Result<()> {
    let address = format!("{}:{}", address, port);
    let mtu = match mtu {
        512 => MTU::MTU512,
        1024 => MTU::MTU1024,
        2048 => MTU::MTU2048,
        4096 => MTU::MTU4096,
        _ => MTU::MTU1024,
    };
    info!("listening for rdma at {}", address);
    let rdma = RdmaBuilder::default().
        set_max_message_length(message_size as usize).
        set_mtu(mtu).
        listen(address.clone()).await?;
    for idx in 0..messages{
        info!("starting listener {}", idx);
        let res = tokio::select! {
            ret = receive(&rdma) => {
                ret
            },
            ret = receive_with_imm(&rdma) => {
                ret
            },
        };
        if let Err(e) = res {
            error!("receive error: {}", e);
        }
        
    }
    info!("done listening");
    Ok(())
}

pub async fn receive(rdma: &Rdma) -> anyhow::Result<()> {
    info!("receiving");
    let lmr = rdma.receive().await?;
    let data = lmr.as_slice();
    info!("received: {:?}", data.len());
    Ok(())
    
}

pub async fn receive_with_imm(rdma: &Rdma) -> anyhow::Result<()> {
    info!("receiving with imm");
    let (lmr, _imm) = rdma.receive_with_imm().await?;
    let data = lmr.as_slice();
    info!("received: {:?}", data.len());
    Ok(())
}