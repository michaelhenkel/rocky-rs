use std::sync::Arc;
use monitor::server::monitor::{
    report_server_client::ReportServerClient,
    Report
};

use tokio::sync::{RwLock, mpsc};
use tonic::Request;

pub struct StatsClient{
    rx: Arc<RwLock<mpsc::Receiver<Command>>>,
    address: String,
    client: Client,
}

impl StatsClient{
    pub fn new(address: String) -> StatsClient{
        let (tx, rx) = mpsc::channel(100);
        StatsClient{
            rx: Arc::new(RwLock::new(rx)),
            address,
            client: Client::new(tx)
        }
    }
    pub fn client(&self) -> Client{
        self.client.clone()
    }

    pub async fn run(&self) -> anyhow::Result<()>{
        let mut grpc_client = ReportServerClient::connect(format!("http://{}", self.address)).await?;
        let rx = self.rx.clone();
        let mut rx = rx.write().await;
        while let Some(command) = rx.recv().await{
            match command{
                Command::Add(report) => {
                    let request = Request::new(report);
                    if let Err(e) = grpc_client.send_report(request).await{
                        log::error!("Error sending report to monitor: {}", e);
                    }
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct Client{
    tx: mpsc::Sender<Command>
}

impl Client{
    pub fn new(tx: mpsc::Sender<Command>) -> Client{
        Client{tx}
    }
    pub async fn add(&self, report: Report) -> anyhow::Result<()>{
        self.tx.send(Command::Add(report)).await?;
        Ok(())
    }
}

pub enum Command{
    Add(Report)
}
