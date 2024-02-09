use std::str::FromStr;
use byte_unit::Byte;
use rocky_rs::listener::listener::{
    listener_client::ListenerClient,
    Operation,
    SendRequest,
    Mtu,
};
use serde::{Deserialize, Serialize};
use clap::Parser;
use config::config::load_config;
pub mod config;

#[derive(Parser, Debug)]
struct BArgs{
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
    volume: Option<String>,
    #[clap(short='n' , long)]
    iterations: Option<u32>,
    #[clap(short, long)]
    mtu: Option<MtuSize>,
}

#[derive(Parser, Debug)]
struct Args{
    #[clap(short, long)]
    cli_config: CliConfig
}

#[derive(Parser, Debug, Clone)]
pub enum CliConfig{
    CliArgs(CliArgs),
    ConfigArgs(ConfigArgs),
}

impl FromStr for CliConfig {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.ends_with(".yaml") {
            Ok(CliConfig::ConfigArgs(ConfigArgs{path: s.to_string()}))
        } else {
            Ok(CliConfig::CliArgs(CliArgs::parse_from(s.split_whitespace())))
        }
    }
}

#[derive(Parser, Debug, Clone, Serialize, Deserialize)]
pub struct CliArgs{
    #[clap(short, long)]
    server: Option<String>,
    #[clap(short, long)]
    initiator: Option<String>,
    #[clap(short, long)]
    server_port: Option<u16>,
    #[clap(short, long)]
    initiator_port: Option<u16>,
    #[clap(value_enum)]
    op: Option<ClientOperation>,
    #[clap(short, long)]
    message_size: Option<String>,
    #[clap(short, long)]
    volume: Option<String>,
    #[clap(short='n' , long)]
    iterations: Option<u32>,
    #[clap(short, long)]
    mtu: Option<MtuSize>,
    #[clap(short, long)]
    config: Option<String>,
}

#[derive(Parser, Debug, Clone)]
pub struct ConfigArgs{
    path: String,
}

impl Into<SendRequest> for CliArgs {
    fn into(self) -> SendRequest {
        let mtu = if let Some(mtu) = self.mtu{
            Mtu::from(mtu).into()
        } else {
            Mtu::Mtu1024
        };
        let message_size = if let Some(message_size) = self.message_size{
            Byte::parse_str(&message_size, true).unwrap().as_u64()
        } else {
            8
        };
        let volume = if let Some(volume) = self.volume{
            let volume = Byte::parse_str(&volume, true).unwrap().as_u64();
            if volume % message_size != 0 {
                panic!("volume must be a multiple of message size")
            }
            volume
        } else {
            message_size
        };
        let iterations = self.iterations.unwrap_or(1);
        SendRequest{
            address: format!("{}:{}", self.server.unwrap(), self.server_port.unwrap()),
            op: Operation::from(self.op.unwrap()).into(),
            message_size,
            volume,
            iterations,
            mtu: mtu.into(),
        }
    }
}

#[derive(Parser, Debug, Clone, Serialize, Deserialize)]
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

#[derive(Parser, Debug, Clone, Serialize, Deserialize)]
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
async fn main() -> anyhow::Result<()> {
    let args = CliArgs::parse();
    if let Some(config) = args.config{
        let config = load_config(&config);
        for job in config{
            for phase in job.phases{
                for operation in phase.operations{
                    let request: SendRequest = operation.clone().into();
                    send(request, operation.initiator.unwrap(), operation.initiator_port.unwrap()).await?;
                }
            }
        }
    } else {
        let request: SendRequest = args.clone().into();
        send(request, args.initiator.unwrap(), args.initiator_port.unwrap()).await?;
    }
    Ok(())
}

async fn send(request: SendRequest, initiator_address: String, initiator_port: u16) -> anyhow::Result<()> {
    let initiator_address = format!("http://{}:{}",initiator_address, initiator_port);
    let mut client = ListenerClient::connect(initiator_address).await?;
    let request = tonic::Request::new(request);
    let response = client.send(request).await?;
    println!("{}", response.get_ref().uuid);
    Ok(())
}
