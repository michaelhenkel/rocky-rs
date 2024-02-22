use log::{info, error};
use tokio_stream::StreamExt;
use tonic::{transport::Server, Request, Response, Status, Streaming};
use clap::Parser;
use monitor::server::monitor::{
    monitor_server_server::{MonitorServer, MonitorServerServer}, 
    Stats, ReceiveReply
};
use monitor::stats_db::stats_db::{StatsDb, StatsDbClient};
use monitor::web_server::web_server::{WebServer, WebServerClient};

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(short, long, default_value = "0.0.0.0:50051")]
    pub address: String,
    #[clap(short, long, default_value = "0.0.0.0:50052")]
    pub grpc_address: String,

}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let args = Args::parse();
    let stats_db = StatsDb::new();
    let web_server = WebServer::new(args.address.clone());
    let monitor = Monitor::new(args.grpc_address.clone(), stats_db.client(), web_server.client());
    let _res = tokio::join!(
        monitor.run(),
        stats_db.run(),
        web_server.run(),
    );
    Ok(())
}

#[derive(Clone)]
pub struct Monitor{
    address: String,
    db_client: StatsDbClient,
    web_server_client: WebServerClient,
}

impl Monitor {
    pub fn new(address: String, db_client: StatsDbClient, web_server_client: WebServerClient) -> Self {
        Monitor{
            address,
            db_client,
            web_server_client,
        }
    }
    pub async fn run(&self) -> anyhow::Result<()> {
        info!("Server listening on {}", self.address);
        Server::builder()
        .add_service(MonitorServerServer::new(self.clone()))
        .serve(self.address.parse()?)
        .await?;
        Ok(())
    }
}

#[tonic::async_trait]
impl MonitorServer for Monitor {
    async fn send_stats(
        &self,
        request: Request<Streaming<Stats>>,
    ) -> Result<Response<ReceiveReply>, Status> {
        info!("Received request");
        let mut stream = request.into_inner();
        while let Some(stats) = stream.next().await {
            let stats = stats?;
            if let Err(e) = self.db_client.insert(stats.clone()).await{
                error!("Error inserting stats: {}", e);
                return Err(Status::internal(e.to_string()));
            }
            if let Err(e) = self.web_server_client.add(stats).await{
                error!("Error sending stats to web server: {}", e);
                return Err(Status::internal(e.to_string()));
            }
        }
        Ok(Response::new(ReceiveReply::default()))
    }
}
