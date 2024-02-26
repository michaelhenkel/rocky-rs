use std::process::Command;
use std::collections::HashMap;
use log::{error, info};
use serde_value::Value;
use serde_json::json;
use crate::monitor_client::monitor_client::Client;
use monitor::server::monitor::{Stats, Meta, Data, Rxe, RxeCounter, RxeHwCounter, Mlx, MlxCounter, MlxHwCounter, PerSec, data};
use gethostname;
use path_resolver::path_trait::PathResolver;

pub struct Collector{
    interfaces: Vec<Interface>,
    freq: u64,
    driver: Driver,
    monitor_client: Option<Client>,
}

#[derive(Clone, Debug)]
struct Interface{
    name: String,
    linux_name: String,
    ports: Vec<String>,
}

#[derive(Clone, Debug)]
pub enum Driver{
    Mlx,
    Rxe,
}

impl std::str::FromStr for Driver{
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err>{
        match s{
            "rxe" => Ok(Driver::Rxe),
            "mlx" => Ok(Driver::Mlx),
            _ => Err("invalid driver".to_string()),
        }
    }
}

impl Driver{
    pub fn get_counters(&self, interface: &str, port: &str, linux_interface: &str) -> Data {
        let mut vars_map = HashMap::new();
        vars_map.insert("interface".to_string(), interface.to_string());
        vars_map.insert("port".to_string(), port.to_string());
        vars_map.insert("linux_interface".to_string(), linux_interface.to_string());
        let mut data = match self{
            Driver::Mlx => {
                let mlx_counter = get_and_set_field_by_name(MlxCounter::default(), vars_map.clone());
                let mlx_hw_counter = get_and_set_field_by_name(MlxHwCounter::default(), vars_map.clone());
                let mut mlx = Mlx::default();
                mlx.mlx_counter = Some(mlx_counter);
                mlx.mlx_hw_counter = Some(mlx_hw_counter);
                let mut monitor_data = Data::default();
                monitor_data.data = Some(data::Data::Mlx(mlx));
                monitor_data

            },
            Driver::Rxe => {
                let rxe_counter = get_and_set_field_by_name(RxeCounter::default(), vars_map.clone());
                let rxe_hw_counter = get_and_set_field_by_name(RxeHwCounter::default(), vars_map.clone());
                let mut rxe = Rxe::default();
                rxe.rxe_counter = Some(rxe_counter);
                rxe.rxe_hw_counter = Some(rxe_hw_counter);
                info!("{:?}", rxe);
                let mut monitor_data = Data::default();
                monitor_data.data = Some(data::Data::Rxe(rxe));
                //info!("{:?}", monitor_data);
                monitor_data
            },
        };
        data.per_sec = Some(PerSec::default());
        data
    }

}

fn read_counter(path: &str) -> u64
{
    let v = match std::fs::read_to_string(path){
        Ok(v) => v,
        Err(_e) => {
            return 0;
        }
    };
    match v.trim().parse::<u64>(){
        Ok(v) => v,
        Err(_e) => {
            0
        }
    }
}

fn get_and_set_field_by_name<T>(data: T, vars_map: HashMap<String, String>) -> T
where
    T: serde::Serialize + serde::de::DeserializeOwned + PathResolver,
{
    let path = data.resolve_path(vars_map);
    let mut map = match serde_value::to_value(data){
        Ok(serde_value::Value::Map(map)) => map,
        _ => {
            panic!("Error converting to value");
        }
    };
    for (k,v) in map.iter_mut(){
        let counter: String = k.clone().deserialize_into().unwrap();
        let path = format!("{}/{}", path, counter);
        let counter_value = read_counter(&path);
        let value: Value = Value::U64(counter_value);
        *v = value;
    }
    let m = json!(map);
    let data: T = serde_json::from_value(m).unwrap();
    data
}

impl Interface{
    pub fn new(name: String) -> Interface{
        let ports: Vec<String> = std::fs::read_dir(format!("/sys/class/infiniband/{}/ports", name)).unwrap()
            .map(|entry| { entry.unwrap().file_name().into_string().unwrap() })
            .collect();        
        if ports.len() > 0 {
            let command = Command::new("rdma").
                arg("link").
                arg("show").
                arg(format!("{}/{}",name, ports[0])).
                output().unwrap();
            let output = String::from_utf8_lossy(&command.stdout);
            let parts: Vec<&str> = output.split(" ").collect();
            let linux_name = parts[parts.len() - 2].to_string();
            Interface{
                name,
                linux_name,
                ports,
            }
        } else {
            Interface{
                name,
                linux_name: "".to_string(),
                ports,
            }
        }
    }
}

impl Collector{
    pub fn new(freq: u64, monitor_client: Option<Client>, driver: Driver) -> Collector{
        let interface_names: Vec<String> = std::fs::read_dir("/sys/class/infiniband").unwrap()
            .map(|entry| entry.unwrap().file_name().into_string().unwrap())
            .collect();
        let mut interfaces = Vec::new();
        for name in interface_names{
            interfaces.push(Interface::new(name));
        }
        Collector{
            interfaces,
            freq,
            monitor_client,
            driver,
        }
    }
    pub async fn run(&self) -> anyhow::Result<()>{
        let interfaces = self.interfaces.clone();
        let mut jh_list = Vec::new();
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        let freq = self.freq;
        let driver = self.driver.clone();
        let jh = tokio::spawn(async move{
            if let Err(e) = count(freq, interfaces, tx, driver).await{
                error!("Error counting: {}", e);
            }
        });
        jh_list.push(jh);
        
        let monitor_client = self.monitor_client.clone();
        let jh = tokio::spawn(async move{
            while let Some(stats) = rx.recv().await{
                if let Some(monitor_client) = monitor_client.clone(){
                    if let Err(e) = monitor_client.send(stats).await{
                        error!("Error sending stats: {}", e);
                    }
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            }
        });
        jh_list.push(jh);
        futures::future::join_all(jh_list).await;
        Ok(())
    }
}

async fn count(freq: u64, interfaces: Vec<Interface>, tx: tokio::sync::mpsc::Sender<Stats>, driver: Driver) -> anyhow::Result<()>{
    let hostname = gethostname::gethostname().to_string_lossy().to_string();
    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(freq));
    let secs: f64 = freq as f64 / 1000 as f64;
    let mut history: HashMap<String,u64> = HashMap::new();
    info!("interfaces: {:?}", interfaces);
    loop{
        interval.tick().await;
        for interface in &interfaces{
            for port in &interface.ports{
                let mut counters = driver.get_counters(&interface.name, port, &interface.linux_name);
                let per_sec = counters.per_sec.as_mut().unwrap();
                let (rcv_packets, rcv_data, xmit_packets, xmit_data) = match counters.data.as_ref().unwrap(){
                    data::Data::Mlx(data) => {
                        (data.mlx_counter.as_ref().unwrap().port_rcv_packets,
                        data.mlx_counter.as_ref().unwrap().port_rcv_data, 
                        data.mlx_counter.as_ref().unwrap().port_xmit_packets,
                        data.mlx_counter.as_ref().unwrap().port_xmit_data)
                    },
                    data::Data::Rxe(data) => {
                        (data.rxe_hw_counter.as_ref().unwrap().rcvd_pkts,
                        data.rxe_counter.as_ref().unwrap().rx_bytes,
                        data.rxe_hw_counter.as_ref().unwrap().sent_pkts,
                        data.rxe_counter.as_ref().unwrap().tx_bytes)
                    }
                };

                let prev_port_rcv_packets_key = format!("{}_{}_rcv_packets", interface.name, port);
                if let Some(prev_port_rcv_packets) = history.get(&prev_port_rcv_packets_key){
                    if prev_port_rcv_packets.clone() < rcv_packets{
                        per_sec.packets_rcv_per_sec = (rcv_packets as f64 - prev_port_rcv_packets.clone() as f64) / secs;
                    }
                }
                history.insert(prev_port_rcv_packets_key, rcv_packets);

                let prev_port_rcv_data_key = format!("{}_{}_rcv_data", interface.name, port);
                if let Some(prev_port_rcv_data) = history.get(&prev_port_rcv_data_key){
                    if prev_port_rcv_data.clone() < rcv_data{
                        per_sec.bytes_rcv_per_sec = (rcv_data as f64 - prev_port_rcv_data.clone() as f64) / secs;
                    }
                }
                history.insert(prev_port_rcv_data_key, rcv_data);

                let prev_port_xmit_packets_key = format!("{}_{}_xmit_packets", interface.name, port);
                if let Some(prev_port_xmit_packets) = history.get(&prev_port_xmit_packets_key){
                    if prev_port_xmit_packets.clone() < xmit_packets{
                        per_sec.packets_xmit_per_sec = (xmit_packets as f64 - prev_port_xmit_packets.clone() as f64) / secs;
                    }
                }
                history.insert(prev_port_xmit_packets_key, xmit_packets);
                
                let prev_port_xmit_data_key = format!("{}_{}_xmit_data", interface.name, port);
                if let Some(prev_port_xmit_data) = history.get(&prev_port_xmit_data_key){
                    if prev_port_xmit_data.clone() < xmit_data{
                        per_sec.bytes_xmit_per_sec = (xmit_data as f64 - prev_port_xmit_data.clone() as f64) / secs;
                    }
                }
                history.insert(prev_port_xmit_data_key, xmit_data);
                
                let stats = Stats{
                    meta: Some(Meta{
                        hostname: hostname.clone(),
                        interface: interface.name.clone(),
                        port: port.clone(),
                    }),
                    data: Some(counters),
                };
                if let Err(e) = tx.send(stats).await{
                    error!("Error sending counters: {}", e);
                }
            }
        }
    }
}

