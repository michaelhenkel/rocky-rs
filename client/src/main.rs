use std::str::FromStr;
use byte_unit::Byte;
use rocky_rs::connection_manager::connection_manager::{
    initiator_connection_client::InitiatorConnectionClient,
    Operation,
    Request,
    Mtu,
    Mode
};
use serde::{Deserialize, Serialize};
use clap::Parser;
use config::config::load_config;
pub mod config;

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
    op: ClientOperation,
    #[clap(value_enum)]
    mode: ClientMode,
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
    #[clap(short, long)]
    cm: Option<bool>,
}

#[derive(Parser, Debug, Clone)]
pub struct ConfigArgs{
    path: String,
}

impl Into<Request> for CliArgs {
    fn into(self) -> Request {
        let mtu = if let Some(mtu) = self.mtu{
            Some(Mtu::from(mtu).into())
        } else {
            None
        };
        let message_size = if let Some(message_size) = self.message_size{
            Some(Byte::parse_str(&message_size, true).unwrap().as_u64())
        } else {
            None
        };
        

        let operation: Operation = self.op.into();
        let mode: Mode = self.mode.into();
        
        Request{
            server_address: self.server.unwrap(),
            server_port: self.server_port.unwrap() as u32,
            uuid: None,
            iterations: self.iterations,
            message_size,
            mtu,
            operation: operation.into(),
            mode: mode.into(),
            cm: self.cm.unwrap_or(false),
        }
    }
}

#[derive(Parser, Debug, Clone, Serialize, Deserialize)]
enum ClientOperation{
    Send,
    Read,
    Write,
    Atomic,
}

impl FromStr for ClientOperation {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "send" => Ok(ClientOperation::Send),
            "read" => Ok(ClientOperation::Read),
            "write" => Ok(ClientOperation::Write),
            "atomic" => Ok(ClientOperation::Atomic),
            _ => Err("invalid operation".to_string()),
        }
    }
}

impl From<ClientOperation> for Operation {
    fn from(op: ClientOperation) -> Self {
        match op {
            ClientOperation::Send => Operation::Send,
            ClientOperation::Read => Operation::Read,
            ClientOperation::Write => Operation::Write,
            ClientOperation::Atomic => Operation::Atomic,
        }
    }
}

#[derive(Parser, Debug, Clone, Serialize, Deserialize)]
enum ClientMode{
    Bw,
    Lat,
}

impl FromStr for ClientMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "bw" => Ok(ClientMode::Bw),
            "lat" => Ok(ClientMode::Lat),
            _ => Err("invalid mode".to_string()),
        }
    }
}

impl From<ClientMode> for Mode {
    fn from(mode: ClientMode) -> Self {
        match mode {
            ClientMode::Bw => Mode::Bw,
            ClientMode::Lat => Mode::Lat,
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
                    let request: Request = operation.clone().into();
                    send(request, operation.initiator.unwrap(), operation.initiator_port.unwrap()).await?;
                }
            }
        }
    } else {
        let request: Request = args.clone().into();
        send(request, args.initiator.unwrap(), args.initiator_port.unwrap()).await?;
    }
    Ok(())
}

async fn send(request: Request, initiator_address: String, initiator_port: u16) -> anyhow::Result<()> {
    let initiator_address = format!("http://{}:{}",initiator_address, initiator_port);
    let mut client = InitiatorConnectionClient::connect(initiator_address).await?;
    let request = tonic::Request::new(request);
    let response = client.initiator(request).await?;
    println!("{}", response.get_ref().uuid);
    Ok(())
}
