use std::{alloc::Layout, io::Write};
use async_rdma::{LocalMr, LocalMrWriteAccess, Rdma, RdmaBuilder, MTU};
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

pub async fn initiate(request: SendRequest) -> anyhow::Result<()> {
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
        id: request.id,
        iterations: request.iterations,
        message_size: request.message_size,
        volume: request.volume,
        mtu
    });
    let response = init_client.init(connect_request).await?;
    let port = response.get_ref().port;
    let address = request.address.split(":").next().unwrap();
    let server_address = format!("{}:{}", address, port);
    info!("connecting to server at {}", server_address);
    let op = Operation::try_from(request.op).unwrap();
    tokio::task::spawn(async move{
        let start = tokio::time::Instant::now();
        let mut stats = Stats::default();
        info!("sending {} iterations with size {}", request.iterations, request.message_size);
        let mut rdma = RdmaBuilder::default().
        set_max_message_length(request.message_size as usize).
        set_mtu(mtu_2).
        connect(server_address.clone()).await.unwrap();
        for _ in 0..request.iterations{
            let sends = request.volume/request.message_size;
            
            rdma = rdma.new_connect(server_address.clone()).await.unwrap();
            info!("allocating {} bytes", request.message_size as usize);
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
            info!("writing done, sending message");
            for _ in 0..sends{
                let res = match op{
                    Operation::Send => {
                        send(&rdma, &lmr).await
                    },
                    Operation::SendWithImm => {
                        send_with_imm(&rdma).await
                    },
                };
                match res {
                    Ok(_) => {
                        stats.messages += 1;
                        stats.total_message_size += request.message_size;
                    },
                    Err(e) => error!("operation error: {}", e),
                }
            }
            let bytes = stats.total_message_size;
            let bits = (bytes*8) as u64;
            let seconds = start.elapsed().as_secs();
            let milliseconds = start.elapsed().as_millis();
            let bps = bits/seconds;
            let kbps = bps/1000;
            let mbps = kbps/1000;
            let gbs = mbps/1000;
            let total_time = if milliseconds > 1000 {
                format!("{} s", seconds)
            } else {
                format!("{} ms", milliseconds)
            };
            stats.total_time = total_time;
            let rate = if gbs > 1 {
                format!("{} Gbps", gbs)
            } else if mbps > 1 {
                format!("{} Mbps", mbps)
            } else if kbps > 1 {
                format!("{} Kbps", kbps)
            } else {
                format!("{} bps", bps)
            };
            stats.rate = rate;

            /*
            let ptr = lmr.as_mut_ptr();
            unsafe { ptr.drop_in_place() };
            drop(buf);
            */
        }
        info!("stats: \n{:#?}", stats);

    });

    Ok(())
}

pub async fn send(rdma: &Rdma, lmr: &LocalMr) -> anyhow::Result<()> {
    rdma.send(&lmr).await?;
    Ok(())
}

pub async fn send_with_imm(rdma: &Rdma) -> anyhow::Result<()> {
    let mut lmr = rdma.alloc_local_mr(Layout::new::<[u8; 8]>())?;
    {
        let mut mr_cursor = lmr.as_mut_slice_cursor();
        let _num = mr_cursor.write(&[1_u8; 4])?;
        let _num = mr_cursor.write(&[1_u8; 4])?;
    }
    rdma.send_with_imm(&lmr, 1_u32).await?;
    Ok(())
}

#[tonic::async_trait]
impl Listener for Initiator {
    async fn send(
        &self,
        request: Request<SendRequest>,
    ) -> Result<Response<SendReply>, Status> {
        let request = request.into_inner();
        
        if let Err(e) = initiate(request).await{
            error!("initiate error: {:?}", e);
            return Err(Status::internal(e.to_string()));
        }
        

        let reply = SendReply{
            message: "all good".to_string(),
        };

        Ok(Response::new(reply))
    }
}

#[derive(Debug, Default)]
struct Stats{
    messages: u32,
    total_message_size: u32,
    total_time: String,
    rate: String,
}

