use std::str::FromStr;
use byte_unit::Byte;
use rocky_rs::connection_manager::connection_manager::{
    initiator_connection_client::InitiatorConnectionClient, 
    stats_manager_client::StatsManagerClient, 
    monitor_client::MonitorClient,
    CounterFilter,
    Mode, Mtu, Operation, ReportList, ReportRequest, Request
};
use tokio_stream::StreamExt;
use serde::{Deserialize, Serialize};
use clap::{Parser, Subcommand};
use config::config::load_config;
pub mod config;
use std::io::{Write, stdout};
use crossterm::{QueueableCommand, cursor, terminal, ExecutableCommand};

#[derive(Parser, Debug, Clone)]
pub enum CliConfig{
    CliArgs(Args),
    ConfigArgs(ConfigArgs),
}

impl FromStr for CliConfig {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.ends_with(".yaml") {
            Ok(CliConfig::ConfigArgs(ConfigArgs{path: s.to_string()}))
        } else {
            Ok(CliConfig::CliArgs(Args::parse_from(s.split_whitespace())))
        }
    }
}

#[derive(Subcommand, Debug, Clone, Serialize, Deserialize)]
enum Commands {
    /// Adds files to myapp
    Run {
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
        #[clap(short='n' , long)]
        iterations: Option<u32>,
        #[clap(short, long)]
        mtu: Option<MtuSize>,
        #[clap(short, long)]
        config: Option<String>,
        #[clap(short, long)]
        cm: Option<bool>,
    },
    Stats {
        #[clap(short, long)]
        address: String,
        #[clap(short, long)]
        port: u16,
        #[clap(subcommand)]
        command: StatsCommands,
    },
    Monitor {
        #[clap(short, long)]
        address: String,
        #[clap(short, long)]
        port: u16,
        #[clap(short, long)]
        filter: Option<MonitorFilter>,
    },
}

#[derive(Parser, Debug, Clone, Serialize, Deserialize)]
pub struct MonitorFilter{
    #[clap(short, long)]
    interface: Option<String>,
    #[clap(short, long)]
    port: Option<u32>,
    #[clap(short, long)]
    counter_list: Vec<String>,
}

impl FromStr for MonitorFilter{
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut filter = MonitorFilter{
            interface: None,
            port: None,
            counter_list: vec![],
        };
        for item in s.split_whitespace(){
            let mut item = item.split("=");
            let key = item.next().unwrap();
            let value = item.next().unwrap();
            match key {
                "interface" => filter.interface = Some(value.to_string()),
                "port" => filter.port = Some(value.parse().unwrap()),
                "counter_list" => filter.counter_list = value.split(",").map(|s| s.to_string()).collect(),
                _ => return Err("invalid filter".to_string()),
            }
        }
        Ok(filter)
    }
}

#[derive(Subcommand, Debug, Clone, Serialize, Deserialize)]
enum StatsCommands {
    Get {
        uuid: String,
        suffix: String,
    },
    List,
    Remove {
        uuid: String,
        suffix: String,
    },
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
    #[clap(short='n' , long)]
    iterations: Option<u32>,
    #[clap(short, long)]
    mtu: Option<MtuSize>,
    #[clap(short, long)]
    config: Option<String>,
    #[clap(short, long)]
    cm: Option<bool>,
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
pub struct Args{
    #[command(subcommand)]
    command: Commands
}

#[derive(Parser, Debug, Clone)]
pub struct ConfigArgs{
    path: String,
}

impl Into<Request> for Commands {
    fn into(self) -> Request {
        match self {
            Commands::Run{server, initiator: _, server_port, initiator_port: _, op, mode, message_size, iterations, mtu, config, cm} => {
                let mtu = if let Some(mtu) = mtu{
                    Some(Mtu::from(mtu).into())
                } else {
                    None
                };
                let message_size = if let Some(message_size) = message_size{
                    Some(Byte::parse_str(&message_size, true).unwrap().as_u64())
                } else {
                    None
                };
                let operation: Operation = op.into();
                let mode: Mode = mode.into();
                Request{
                    server_address: server.unwrap(),
                    server_port: server_port.unwrap() as u32,
                    uuid: None,
                    iterations,
                    message_size,
                    mtu,
                    operation: operation.into(),
                    mode: mode.into(),
                    cm: cm.unwrap_or(false),
                }
            },
            _ => panic!("invalid command"),
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
    let args = Args::parse();
    match args.clone().command {
        Commands::Run{server: _, initiator, server_port: _, initiator_port, op: _, mode: _, message_size: _, iterations: _, mtu: _, config, cm: _} => {
            if let Some(config) = config{
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
                let request: Request = args.command.into();
                send(request, initiator.unwrap(), initiator_port.unwrap()).await?;
            }
        },
        Commands::Stats{address, port, command} => {
            match command {
                StatsCommands::Get{uuid, suffix} => {
                    stats_get(uuid, suffix, address, port).await?;
                },
                StatsCommands::List => {
                    stats_list(address, port).await?;
                },
                StatsCommands::Remove{uuid, suffix} => {
                    stats_remove(uuid, suffix, address, port).await?;
                },
            }
        },
        Commands::Monitor { address, port, filter } => {
            let address = format!("http://{}:{}",address, port);
            let mut client = MonitorClient::connect(address).await?;
            let counter_filter = CounterFilter{
                interface: filter.as_ref().map(|f| f.interface.clone()).unwrap_or_default(),
                port: filter.as_ref().map(|f| f.port).unwrap_or_default(),
                counter_list: filter.map(|f| f.counter_list).unwrap_or_default(),
            };
            let request = tonic::Request::new(counter_filter);
            let mut stream = client.monitor_stream(request).await?.into_inner();
            let mut stdout = stdout();
            stdout.execute(cursor::Hide).unwrap();
            while let Some(item) = stream.next().await {
                stdout.queue(cursor::SavePosition).unwrap();
                stdout.write_all(format!("{:#?}", item).as_bytes()).unwrap();
                stdout.queue(cursor::RestorePosition).unwrap();
                stdout.flush().unwrap();
                stdout.queue(cursor::RestorePosition).unwrap();
                stdout.queue(terminal::Clear(terminal::ClearType::FromCursorDown)).unwrap();
            }
        }
    }
    Ok(())
}

async fn send(request: Request, initiator_address: String, initiator_port: u16) -> anyhow::Result<()> {
    let initiator_address = format!("http://{}:{}",initiator_address, initiator_port);
    let mut client = InitiatorConnectionClient::connect(initiator_address).await?;
    let response = client.initiator(request).await?;
    println!("{}", response.get_ref().uuid);
    Ok(())
}

async fn stats_get(uuid: String, suffix: String, address: String, port: u16) -> anyhow::Result<()> {
    let address = format!("http://{}:{}",address, port);
    let mut client = StatsManagerClient::connect(address).await?;
    let request = ReportRequest{
        uuid,
        suffix,
    };
    let response = client.get_report(request).await?;
    println!("{:?}", response.get_ref());
    Ok(())
}

async fn stats_list(address: String, port: u16) -> anyhow::Result<()> {
    let address = format!("http://{}:{}",address, port);
    let mut client = StatsManagerClient::connect(address).await?;
    let response = client.list_report(()).await?;
    let report: ReportList = response.into_inner();
    println!("{:#?}", report);
    Ok(())
}

async fn stats_remove(uuid: String, suffix: String, address: String, port: u16) -> anyhow::Result<()> {
    let address = format!("http://{}:{}",address, port);
    let mut client = StatsManagerClient::connect(address).await?;
    let request = ReportRequest{
        uuid,
        suffix,
    };
    let response = client.delete_report(request).await?;
    println!("{:?}", response.get_ref());
    Ok(())
}
