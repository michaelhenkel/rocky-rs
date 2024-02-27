use std::{fmt::Display, sync::Arc};

use regex::Regex;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use monitor::server::monitor::{
    Report as GrpcReport,
    TestInfo as GrpcTestInfo,
    BwResults as GrpcBwResults
};
use crate::stats_client::{stats_client::Client as StatsClient};

pub struct StatsManager{
    client: Client,
    rx: Arc<RwLock<tokio::sync::mpsc::Receiver<Command>>>,
    stats_client: Option<StatsClient>,
}

impl StatsManager{
    pub fn new(stats_client: Option<StatsClient>) -> StatsManager{
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        StatsManager{
            client: Client::new(tx),
            rx: Arc::new(RwLock::new(rx)),
            stats_client,
        }
    }
    pub fn client(&self) -> Client{
        self.client.clone()
    }
    pub async fn run(&self) -> anyhow::Result<()>{
        let rx = self.rx.clone();
        let mut rx = rx.write().await;

        while let Some(command) = rx.recv().await{
            match command{
                Command::Add{uuid, suffix} => {
                    if let Some(stats_client) = &self.stats_client{
                        let report = Report::new(&uuid, &suffix);
                        let grpc_report: GrpcReport = report.clone().into();
                        if let Err(e) = stats_client.add(grpc_report).await{
                            log::error!("Error sending report to monitor: {}", e);
                        }
                    }
                },
            }
        }
        Ok(())
    }
}

pub enum Command{
    Add{
        uuid: String,
        suffix: String,
    },
}

#[derive(Clone)]
pub struct Client{
    tx: tokio::sync::mpsc::Sender<Command>
}
impl Client{
    pub fn new(tx: tokio::sync::mpsc::Sender<Command>) -> Client{
        Client{tx}
    }
    pub async fn add(&self, uuid: String, suffix: String) -> anyhow::Result<()>{
        self.tx.send(Command::Add{uuid, suffix}).await?;
        Ok(())
    }
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Report{
    hostname: Option<String>,
    uuid: Option<String>,
    test_info: TestInfo,
    results: BwResults,
}

impl Report{
    pub fn new(uuid: &str, suffix: &str) -> Report{
        let hostname = hostname::get().unwrap().into_string().unwrap();
        let output = std::fs::read_to_string(format!("/tmp/{}-{}.json",uuid, suffix)).unwrap();
        let quoted_string = quote_strings(&output);
        let mut results: Report = serde_json::from_str(&quoted_string).unwrap();
        results.uuid = Some(uuid.to_string());
        results.hostname = Some(hostname.to_string());
        results
    }
}

impl Into<GrpcReport> for Report{
    fn into(self) -> GrpcReport{
        let grpc_test_info = GrpcTestInfo{
            test: self.test_info.test,
            dual_port: self.test_info.dual_port,
            device: self.test_info.device,
            number_of_qps: self.test_info.number_of_qps,
            transport_type: self.test_info.transport_type,
            connection_type: self.test_info.connection_type,
            using_srq: self.test_info.using_srq,
            pci_relax_order: self.test_info.pci_relax_order,
            ibv_wr_api: self.test_info.ibv_wr_api,
            tx_depth: self.test_info.tx_depth,
            rx_depth: self.test_info.rx_depth,
            cq_moderation: self.test_info.cq_moderation,
            mtu: self.test_info.mtu,
            link_type: self.test_info.link_type,
            gid_index: self.test_info.gid_index,
            max_inline_data: self.test_info.max_inline_data,
            rdma_cm_qps: self.test_info.rdma_cm_qps,
            data_ex_method: self.test_info.data_ex_method,
        };
        let grpc_bw_results = GrpcBwResults{
            msg_size: self.results.msg_size,
            n_iterations: self.results.n_iterations,
            bw_peak: self.results.bw_peak as f64,
            bw_average: self.results.bw_average as f64,
            msg_rate: self.results.msg_rate as f64,
        };
        GrpcReport{
            hostname: self.hostname.unwrap(),
            uuid: self.uuid.unwrap(),
            test_info: Some(grpc_test_info),
            bw_results: Some(grpc_bw_results),
        }
    }
}

#[derive(Debug, Deserialize, Clone, Serialize)]
struct TestInfo{
    test: String,
    #[serde(rename(deserialize = "Dual_port"))]
    dual_port: String,
    #[serde(rename(deserialize = "Device"))]
    device: String,
    #[serde(rename(deserialize = "Number_of_qps"))]
    number_of_qps: u32,
    #[serde(rename(deserialize = "Transport_type"))]
    transport_type: String,
    #[serde(rename(deserialize = "Connection_type"))]
    connection_type: String,
    #[serde(rename(deserialize = "Using_SRQ"))]
    using_srq: String,
    #[serde(rename(deserialize = "PCIe_relax_order"))]
    pci_relax_order: String,
    #[serde(rename(deserialize = "ibv_wr_API"))]
    ibv_wr_api: String,
    #[serde(rename(deserialize = "TX_depth"))]
    tx_depth: Option<u32>,
    #[serde(rename(deserialize = "RX_depth"))]
    rx_depth: Option<u32>,
    #[serde(rename(deserialize = "CQ_Moderation"))]
    cq_moderation: u32,
    #[serde(rename(deserialize = "Mtu"))]
    mtu: u32,
    #[serde(rename(deserialize = "Link_type"))]
    link_type: String,
    #[serde(rename(deserialize = "GID_index"))]
    gid_index: u32,
    #[serde(rename(deserialize = "Max_inline_data"))]
    max_inline_data: u32,
    #[serde(rename(deserialize = "rdma_cm_QPs"))]
    rdma_cm_qps: String,
    #[serde(rename(deserialize = "Data_ex_method"))]
    data_ex_method: String,
}


#[derive(Debug, Deserialize, Clone, Serialize)]
struct BwResults{
    #[serde(rename(deserialize = "MsgSize"))]
    msg_size: u32,
    #[serde(rename(deserialize = "n_iterations"))]
    n_iterations: u32,
    #[serde(rename(deserialize = "BW_peak"))]
    bw_peak: f32,
    #[serde(rename(deserialize = "BW_average"))]
    bw_average: f32,
    #[serde(rename(deserialize = "MsgRate"))]
    msg_rate: f32,
}

impl Display for Report{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Test Info: {}\nResults: {}", self.test_info, self.results)
    }
}

impl Display for TestInfo{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Test Info: {}", self)
    }
}

impl Display for BwResults{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Results: {}", self)
    }
}

fn quote_strings(input: &str) -> String {
    let re = Regex::new(r"\b([a-zA-Z]\w*)\b").unwrap();
    let result = re.replace_all(input, "\"$1\"");
    let re = Regex::new(r#""""#).unwrap(); 
    let result = re.replace_all(&result, "\"");
    let re = Regex::new(r",\s*([\]}])").unwrap();
    let result = re.replace_all(&result, "$1");
    result.into_owned()
}
