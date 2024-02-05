use std::{alloc::Layout, io::Write};
use async_rdma::{LocalMr, LocalMrWriteAccess, MrAccess, Rdma, RdmaBuilder, MTU};
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
    info!("initiating connection to server at {}", init_address);
    let mtu: Mtu = Mtu::try_from(request.mtu).unwrap();
    let (mtu, mtu_2) = match mtu {
        Mtu::Mtu512 => (512, MTU::MTU512),
        Mtu::Mtu1024 => (1024, MTU::MTU1024),
        Mtu::Mtu2048 => (2048, MTU::MTU2048),
        Mtu::Mtu4096 => (4096, MTU::MTU4096)
    };
    let mut init_client = ConnectionClient::connect(init_address.clone()).await?;
    info!("connected to server at {}", init_address);
    let mut message_fragments = request.message_volume / request.message_size;
    if request.message_volume % request.message_size > 0 {
        message_fragments += 1;
    }
    info!("message volume: {}", request.message_volume);
    info!("message size: {}", request.message_size);
    info!("message fragments: {}", message_fragments);

    let connect_request = tonic::Request::new(ConnectRequest{
        id: request.id,
        messages: message_fragments,
        message_size: request.message_size,
        mtu
    });
    let response = init_client.init(connect_request).await?;
    info!("response: {:?}", response.get_ref());
    let port = response.get_ref().port;
    let address = request.address.split(":").next().unwrap();
    let server_address = format!("{}:{}", address, port);
    info!("connecting to server at {}", server_address);
    let op = Operation::try_from(request.op).unwrap();
    tokio::task::spawn(async move{
        let b = RdmaBuilder::default().
            set_max_message_length(request.message_size as usize).
            set_mtu(mtu_2).
            connect(server_address.clone());

        let rdma = match b.await{
            Ok(rdma) => rdma,
            Err(e) => {
                error!("rdma connect error: {}", e);
                return;
            }
        };
        
        let mut stats = Stats::default();
        let start = tokio::time::Instant::now();
        let buf = vec![0_u8; request.message_volume as usize];
        let buf = buf.as_slice();

        for (idx, msg_size) in partition(request.message_volume, request.message_size).iter().enumerate(){
            info!("message size: {}", msg_size);
            let layout = Layout::from_size_align(*msg_size as usize, 1).unwrap();
            let mut lmr = rdma.alloc_local_mr(layout).unwrap();
            let start_offset = idx as usize * request.message_size as usize;
            let end_offset = start_offset + *msg_size as usize;
            let _num = lmr.as_mut_slice().write(&buf[start_offset..end_offset]).unwrap();
            info!("lmr size: {}", lmr.length());

            let res = match op{
                Operation::Send => {
                    info!("send operation");
                    send(&rdma, &lmr).await
                },
                Operation::SendWithImm => {
                    info!("send_with_imm operation");
                    send_with_imm(&rdma, request.message_size, request.messages).await
                },
            };
            match res {
                Ok(_) => {
                    stats.messages += stats.messages + 1;
                    stats.total_bytes += stats.total_bytes + *msg_size;
                    stats.total_time += stats.total_time + start.elapsed().as_millis() as u32;
                },
                Err(e) => error!("operation error: {}", e),
            }
        }

        info!("stats: {:#?}", stats);
    });

    Ok(())
}

//pub async fn send(rdma: &Rdma, message_size: u32, messages: u32) -> anyhow::Result<()> {
pub async fn send(rdma: &Rdma, lmr: &LocalMr) -> anyhow::Result<()> {
    rdma.send(lmr).await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
    Ok(())
}

pub async fn send_with_imm(rdma: &Rdma, message_size: u32, messages: u32) -> anyhow::Result<()> {
    let layout = Layout::from_size_align(message_size as usize, 1).unwrap();
    let mut lmr = rdma.alloc_local_mr(layout)?;
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

//fn partition takes a message volume and message size and returns a vector of message fragments. If there is a remainder, the last fragment will be smaller than the rest.
fn partition(message_volume: u32, message_size: u32) -> Vec<u32> {
    let message_fragments = message_volume / message_size;
    let mut fragments = vec![message_size; message_fragments as usize];
    let remainder = message_volume % message_size;
    if remainder > 0 {
        fragments.push(remainder);
    }
    fragments
}

#[derive(Default, Debug)]
struct Stats{
    messages: u32,
    total_bytes: u32,
    total_time: u32,
    mbps: f64,
}

