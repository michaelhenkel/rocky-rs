use std::{collections::HashMap, pin::Pin, sync::Arc};
use log::{error,info};
use prometheus::{GaugeVec, Opts, Registry};
use tokio::sync::{mpsc, RwLock};
use actix_web::{get, App, HttpServer, Responder};
use actix_web_prom::PrometheusMetricsBuilder;

use crate::server::monitor::Stats;

const METRICS_LIST: &'static [&'static str] = &[
    "unicast_xmit_packets",
    "unicast_rcv_packets",
    "port_xmit_wait",
    "port_xmit_packets",
    "port_rcv_packets",
    "port_rcv_errors",
    "port_rcv_data",
    "multicast_xmit_packets",
    "multicast_rcv_packets",
    "rx_write_requests",
    "rx_read_requests",
    "rx_atomic_requests",
    "resp_cqe_errors",
    "req_cqe_errors",
    "resp_cqe_fl",
    "out_of_sequence",
    "out_of_buffer",
    "local_ack_timeout_errors",
    "implied_nak_seq_errors",
    "duplicate_request",
    "bytes_xmit_per_sec",
    "packets_xmit_per_sec",
    "bytes_rcv_per_sec",
    "packets_rcv_per_sec",
];

#[get("/")]
async fn index() -> impl Responder {
    "Hello, World!"
}

pub struct WebServer{
    address: String,
    rx: Arc<RwLock<mpsc::Receiver<WebServerCommand>>>,
    client: WebServerClient,
}

impl WebServer {
    pub fn new(address: String) -> Self {
        let (tx, rx) = mpsc::channel(1);
        WebServer{
            address,
            rx: Arc::new(RwLock::new(rx)),
            client: WebServerClient::new(tx),
        }
    }
    pub fn client(&self) -> WebServerClient {
        self.client.clone()
    }
    pub async fn run(&self) -> anyhow::Result<()> {
        let (tx, rx) = mpsc::channel(1);
        let _res = tokio::join!(
            self.server(rx),
            self.receive_stats(tx),
        );
        Ok(())
    }
    pub async fn server(&self, mut stats_rx: mpsc::Receiver<Stats>) -> anyhow::Result<()> {
        let mut prometheus = PrometheusMetricsBuilder::new("api")
            .endpoint("/metrics")
            .build()
            .unwrap();

    
        let (reg, gauge_map) = setup_metrics();
        
        prometheus.registry = reg.clone();

        tokio::spawn(async move {
            while let Some(stats) = stats_rx.recv().await {
                let data = stats.clone().data.unwrap();
                let meta = stats.clone().meta.unwrap();
                let hostname = meta.hostname.clone();
                let interface = meta.interface.clone();
                let port = meta.port.clone();
                Pin::new(&mut gauge_map.get("unicast_xmit_packets").unwrap()).with_label_values(&[&hostname, &interface, &port]).set(data.unicast_xmit_packets as f64);
                Pin::new(&mut gauge_map.get("unicast_rcv_packets").unwrap()).with_label_values(&[&hostname, &interface, &port]).set(data.unicast_rcv_packets as f64);
                Pin::new(&mut gauge_map.get("port_xmit_wait").unwrap()).with_label_values(&[&hostname, &interface, &port]).set(data.port_xmit_wait as f64);
                Pin::new(&mut gauge_map.get("port_xmit_packets").unwrap()).with_label_values(&[&hostname, &interface, &port]).set(data.port_xmit_packets as f64);
                Pin::new(&mut gauge_map.get("port_rcv_packets").unwrap()).with_label_values(&[&hostname, &interface, &port]).set(data.port_rcv_packets as f64);
                Pin::new(&mut gauge_map.get("port_rcv_errors").unwrap()).with_label_values(&[&hostname, &interface, &port]).set(data.port_rcv_errors as f64);
                Pin::new(&mut gauge_map.get("port_rcv_data").unwrap()).with_label_values(&[&hostname, &interface, &port]).set(data.port_rcv_data as f64);
                Pin::new(&mut gauge_map.get("multicast_xmit_packets").unwrap()).with_label_values(&[&hostname, &interface, &port]).set(data.multicast_xmit_packets as f64);
                Pin::new(&mut gauge_map.get("multicast_rcv_packets").unwrap()).with_label_values(&[&hostname, &interface, &port]).set(data.multicast_rcv_packets as f64);
                Pin::new(&mut gauge_map.get("rx_write_requests").unwrap()).with_label_values(&[&hostname, &interface, &port]).set(data.rx_write_requests as f64);
                Pin::new(&mut gauge_map.get("rx_read_requests").unwrap()).with_label_values(&[&hostname, &interface, &port]).set(data.rx_read_requests as f64);
                Pin::new(&mut gauge_map.get("rx_atomic_requests").unwrap()).with_label_values(&[&hostname, &interface, &port]).set(data.rx_atomic_requests as f64);
                Pin::new(&mut gauge_map.get("resp_cqe_errors").unwrap()).with_label_values(&[&hostname, &interface, &port]).set(data.resp_cqe_errors as f64);
                Pin::new(&mut gauge_map.get("req_cqe_errors").unwrap()).with_label_values(&[&hostname, &interface, &port]).set(data.req_cqe_errors as f64);
                Pin::new(&mut gauge_map.get("resp_cqe_fl").unwrap()).with_label_values(&[&hostname, &interface, &port]).set(data.resp_cqe_fl as f64);
                Pin::new(&mut gauge_map.get("out_of_sequence").unwrap()).with_label_values(&[&hostname, &interface, &port]).set(data.out_of_sequence as f64);
                Pin::new(&mut gauge_map.get("out_of_buffer").unwrap()).with_label_values(&[&hostname, &interface, &port]).set(data.out_of_buffer as f64);
                Pin::new(&mut gauge_map.get("local_ack_timeout_errors").unwrap()).with_label_values(&[&hostname, &interface, &port]).set(data.local_ack_timeout_errors as f64);
                Pin::new(&mut gauge_map.get("implied_nak_seq_errors").unwrap()).with_label_values(&[&hostname, &interface, &port]).set(data.implied_nak_seq_errors as f64);
                Pin::new(&mut gauge_map.get("duplicate_request").unwrap()).with_label_values(&[&hostname, &interface, &port]).set(data.duplicate_request as f64);
                Pin::new(&mut gauge_map.get("bytes_xmit_per_sec").unwrap()).with_label_values(&[&hostname, &interface, &port]).set(data.bytes_xmit_per_sec as f64);
                Pin::new(&mut gauge_map.get("packets_xmit_per_sec").unwrap()).with_label_values(&[&hostname, &interface, &port]).set(data.packets_xmit_per_sec as f64);
                Pin::new(&mut gauge_map.get("bytes_rcv_per_sec").unwrap()).with_label_values(&[&hostname, &interface, &port]).set(data.bytes_rcv_per_sec as f64);
                Pin::new(&mut gauge_map.get("packets_rcv_per_sec").unwrap()).with_label_values(&[&hostname, &interface, &port]).set(data.packets_rcv_per_sec as f64);
            }
        });

        info!("webserver listening on {}", self.address);
        HttpServer::new(move || {
            App::new()
                .wrap(prometheus.clone())
                .service(index)
        })
        .bind(self.address.clone())?
        .run()
        .await?;
        Ok(())
    }


    pub async fn receive_stats(&self, tx: mpsc::Sender<Stats>) -> anyhow::Result<()> {
        let mut rx = self.rx.write().await;
        while let Some(command) = rx.recv().await {
            match command {
                WebServerCommand::Add(stats) => {
                    tx.send(stats).await?;
                }
            }
        }
        info!("Web server receive stats loop ended");
        Ok(())
    }
}

fn setup_metrics() -> (Registry, HashMap<String, GaugeVec>){
    let registry = Registry::new();
    let mut gauge_map = HashMap::new();
    for metric in METRICS_LIST.iter() {
        let opts = Opts::new(metric.to_string(), metric.to_string());
        let gauge = GaugeVec::new(opts, &["hostname","interface","port"]).unwrap();
        registry.register(Box::new(gauge.clone())).unwrap();
        gauge_map.insert(metric.to_string(), gauge);
    }
    (registry, gauge_map)

}

#[derive(Clone)]
pub struct WebServerClient{
    tx: mpsc::Sender<WebServerCommand>,
}

impl WebServerClient{
    pub fn new(tx: mpsc::Sender<WebServerCommand>) -> Self {
        WebServerClient{
            tx,
        }
    }
    pub async fn add(&self, stats: Stats) -> anyhow::Result<()> {
        if let Err(e) = self.tx.send(WebServerCommand::Add(stats)).await{
            error!("Error sending stats to web server: {}", e);
            return Err(anyhow::anyhow!("Error sending stats to web server: {}", e));
        }
        Ok(())
    }

}

pub enum WebServerCommand{
    Add(Stats),
}