use std::str::FromStr;
use byte_unit::Byte;
use rocky_rs::listener::listener::{
    listener_client::ListenerClient,
    Operation,
    SendRequest,
    Mtu,
};
use clap::Parser;

#[derive(Parser, Debug)]
struct Args{
    #[clap(short, long)]
    id: u32,
    #[clap(short, long)]
    server: String,
    #[clap(short, long)]
    initiator: String,
    #[clap(short, long)]
    server_port: u16,
    #[clap(short, long)]
    initiator_port: u16,
    #[clap(value_enum)]
    op: ClientOperation,
    #[clap(short, long)]
    message_size: Option<String>,
    #[clap(short, long)]
    message_volume: Option<String>,
    #[clap(short, long)]
    mtu: Option<MtuSize>,
}

impl Into<SendRequest> for Args {
    fn into(self) -> SendRequest {
        let mtu = if let Some(mtu) = self.mtu{
            Mtu::from(mtu).into()
        } else {
            Mtu::Mtu1024
        };
        let message_size = if let Some(message_size) = self.message_size{
            Byte::parse_str(&message_size, true).unwrap().as_u64() as u32
        } else {
            8
        };
        let message_volume = if let Some(message_volume) = self.message_volume{
            Byte::parse_str(&message_volume, true).unwrap().as_u64() as u32
        } else {
            1
        };
        SendRequest{
            id: self.id,
            address: format!("{}:{}", self.server, self.server_port),
            op: Operation::from(self.op).into(),
            message_size,
            message_volume,
            mtu: mtu.into(),
        }
    }
}

#[derive(Parser, Debug, Clone)]
enum ClientOperation{
    Send,
    SendWithImm,
}

impl FromStr for ClientOperation {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "send" => Ok(ClientOperation::Send),
            "send_with_imm" => Ok(ClientOperation::SendWithImm),
            _ => Err("invalid operation".to_string()),
        }
    }
}

impl From<ClientOperation> for Operation {
    fn from(op: ClientOperation) -> Self {
        match op {
            ClientOperation::Send => Operation::Send,
            ClientOperation::SendWithImm => Operation::SendWithImm,
        }
    }
}

#[derive(Parser, Debug, Clone)]
enum MtuSize{
    Mtu512,
    Mtu1024,
    Mtu2048,
    Mtu4096,
}

impl FromStr for MtuSize {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "512" => Ok(MtuSize::Mtu512),
            "1024" => Ok(MtuSize::Mtu1024),
            "2048" => Ok(MtuSize::Mtu2048),
            "4096" => Ok(MtuSize::Mtu4096),
            _ => Err("invalid mtu".to_string()),
        }
    }
}

impl From<MtuSize> for Mtu {
    fn from(mtu: MtuSize) -> Self {
        match mtu {
            MtuSize::Mtu512 => Mtu::Mtu512,
            MtuSize::Mtu1024 => Mtu::Mtu1024,
            MtuSize::Mtu2048 => Mtu::Mtu2048,
            MtuSize::Mtu4096 => Mtu::Mtu4096,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let initiator_address = format!("http://{}:{}",args.initiator, args.initiator_port);
    let mut client = ListenerClient::connect(initiator_address).await?;
    let request = tonic::Request::new(args.into());
    let response = client.send(request).await?;
    println!("RESPONSE={:?}", response);
    Ok(())
}
