use log::{error, info};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::Request;
use monitor::server::monitor::{monitor_server_client::MonitorServerClient, Stats};

pub struct MonitorClient {
    pub client: Client,
    pub rx: mpsc::Receiver<Stats>,
    pub address: String
}

impl MonitorClient {
    pub fn new(address: String) -> Self {
        let (tx, rx) = mpsc::channel(100);
        MonitorClient {
            client: Client::new(tx),
            rx,
            address,
        }
    }
    pub fn client(&self) -> Client {
        self.client.clone()
    }
    pub async fn run(self) -> anyhow::Result<()> {
        info!("connecting monitor client to {}", self.address.clone());
        let mut client = MonitorServerClient::connect(format!("http://{}", self.address)).await?;
        let output_stream = ReceiverStream::new(self.rx);
        let request = Request::new(output_stream);
        if let Err(e) = client.send_stats(request).await{
            error!("monitor client error: {}", e.to_string());
        }
        info!("monitor client done");
        Ok(())
    }
}

#[derive(Clone)]
pub struct Client{
    pub tx: mpsc::Sender<Stats>,
}

impl Client {
    pub fn new(tx: mpsc::Sender<Stats>) -> Self {
        Client {
            tx,
        }
    }
    pub async fn send(&self, stats: Stats) -> anyhow::Result<()> {
        self.tx.send(stats).await?;
        Ok(())
    }
}