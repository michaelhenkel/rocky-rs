use std::{collections::{HashMap, HashSet}, pin::Pin, sync::Arc};
use log::{error,info};
use prometheus::{GaugeVec, Opts, Registry};
use tokio::sync::{mpsc, RwLock};
use actix_web::{get, App, HttpServer, Responder};
use actix_web_prom::PrometheusMetricsBuilder;


use crate::server::monitor::{data, BwResults, InterfaceStats, MlxCounter, MlxHwCounter, PerSec, Report, RxeCounter, RxeHwCounter};

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
    pub async fn server(&self, mut stats_rx: mpsc::Receiver<InterfaceStatsReport>) -> anyhow::Result<()> {
        let mut prometheus = PrometheusMetricsBuilder::new("api")
            .endpoint("/metrics")
            .build()
            .unwrap();

        let (reg, gauge_map) = setup_metrics();
        prometheus.registry = reg.clone();

        tokio::spawn(async move {
            while let Some(interface_stats_report) = stats_rx.recv().await {
                match interface_stats_report{
                    InterfaceStatsReport::InterfaceStats(interface_stats) => {
                        let hostname = interface_stats.hostname.clone();
                        for (interface, port_stats) in &interface_stats.port_stats{
                            for (port, data) in &port_stats.data{
                                let per_sec = data.per_sec.as_ref().unwrap();
                                
                                let _elapsed: u128 = ((data.elapsed.as_ref().unwrap().high as u128) << 64) | (data.elapsed.as_ref().unwrap().low as u128);
                                match data.data.as_ref().unwrap(){
                                    data::Data::Mlx(data) => {
                                        let values = get_data_values(&data.mlx_counter.as_ref().unwrap());
                                        for (k,v) in values.iter(){
                                            Pin::new(&mut gauge_map.get(k).unwrap()).with_label_values(&[&hostname, &interface, &port]).set(*v);
                                        }
                                        let values = get_data_values(&data.mlx_hw_counter.as_ref().unwrap());
                                        for (k,v) in values.iter(){
                                            Pin::new(&mut gauge_map.get(k).unwrap()).with_label_values(&[&hostname, &interface, &port]).set(*v);
                                        }
                                    },
                                    data::Data::Rxe(data) => {
                                        let values = get_data_values(&data.rxe_counter.as_ref().unwrap());
                                        for (k,v) in values.iter(){
                                            Pin::new(&mut gauge_map.get(k).unwrap()).with_label_values(&[&hostname, &interface, &port]).set(*v);
                                        }
                                        let values = get_data_values(&data.rxe_hw_counter.as_ref().unwrap());
                                        for (k,v) in values.iter(){
                                            Pin::new(&mut gauge_map.get(k).unwrap()).with_label_values(&[&hostname, &interface, &port]).set(*v);
                                        }
                                    },
                
                                }
                                let values = get_data_values(&per_sec);
                                for (k,v) in values.iter(){
                                    Pin::new(&mut gauge_map.get(k).unwrap()).with_label_values(&[&hostname, &interface, &port]).set(*v);
                                }
        
                            }
                        }
                    },
                    InterfaceStatsReport::Report(report) => {
                        let hostname = report.hostname.clone();
                        let values = get_data_values(&report.bw_results.as_ref().unwrap());
                        for (k,v) in values.iter(){
                            Pin::new(&mut gauge_map.get(k).unwrap()).with_label_values(&[&hostname]).set(*v);
                        }
                        info!("Received report");
                    }
                }

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


    pub async fn receive_stats(&self, tx: mpsc::Sender<InterfaceStatsReport>) -> anyhow::Result<()> {
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
    let mut fields_map = HashSet::new();
    let rxe_data = RxeHwCounter::default();
    get_data_fields(&rxe_data, &mut fields_map);
    let rxe_data = RxeCounter::default();
    get_data_fields(&rxe_data, &mut fields_map);
    let mlx_data = MlxCounter::default();
    get_data_fields(&mlx_data, &mut fields_map);
    let mlx_data = MlxHwCounter::default();
    get_data_fields(&mlx_data, &mut fields_map);
    let per_sec_data = PerSec::default();
    get_data_fields(&per_sec_data, &mut fields_map);
    for field in fields_map.iter(){
        let opts = Opts::new(field.to_string(), field.to_string());
        let gauge = GaugeVec::new(opts, &["hostname","interface","port"]).unwrap();
        registry.register(Box::new(gauge.clone())).unwrap();
        gauge_map.insert(field.to_string(), gauge);
    }
    let mut fields_map = HashSet::new();
    let bw_results = BwResults::default();
    get_data_fields(&bw_results, &mut fields_map);
    for field in fields_map.iter(){
        let opts = Opts::new(field.to_string(), field.to_string());
        let gauge = GaugeVec::new(opts, &["hostname"]).unwrap();
        registry.register(Box::new(gauge.clone())).unwrap();
        gauge_map.insert(field.to_string(), gauge);
    }
    (registry, gauge_map)
}

fn get_data_fields<T>(data: &T, fields_map: &mut HashSet<String>)
where
    T: serde::Serialize,
{
    let map = match serde_value::to_value(data){
        Ok(serde_value::Value::Map(map)) => map,
        _ => {
            panic!("Error converting to value");
        }
    };
    for (k,_) in map.iter(){
        let k: String = k.clone().deserialize_into().unwrap();
        fields_map.insert(k);
    }
}

fn get_data_values<T>(data: &T) -> HashMap::<String, f64>
where
    T: serde::Serialize,
{
    let mut values = HashMap::new();
    let map = match serde_value::to_value(data){
        Ok(serde_value::Value::Map(map)) => map,
        _ => {
            panic!("Error converting to value");
        }
    };
    for (k,v) in map.iter(){
        let v: f64 = v.clone().deserialize_into().unwrap();
        let k: String = k.clone().deserialize_into().unwrap();
        values.insert(k, v);
    }
    values
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
    pub async fn add(&self, stats_report: InterfaceStatsReport) -> anyhow::Result<()> {
        if let Err(e) = self.tx.send(WebServerCommand::Add(stats_report)).await{
            error!("Error sending stats to web server: {}", e);
            return Err(anyhow::anyhow!("Error sending stats to web server: {}", e));
        }
        Ok(())
    }

}
pub enum WebServerCommand{
    Add(InterfaceStatsReport),

}

pub enum InterfaceStatsReport{
    InterfaceStats(InterfaceStats),
    Report(Report),
}