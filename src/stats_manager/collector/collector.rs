use std::process::Command;
use std::{collections::HashMap, sync::Arc};
use log::{error, info};
use serde::de::value;
use serde_value::Value;
use tokio::sync::RwLock;
use crate::monitor_client::monitor_client::Client;
use monitor::server::monitor::{Stats, Meta, Data, RxeData, MlxData, data};
use gethostname;
use ::phf::{Map, phf_map};

const MLX_COUNTERS: &'static [&'static str] = &[
    "VL15_dropped",
    "excessive_buffer_overrun_errors",
    "link_downed",
    "link_error_recovery",
    "local_link_integrity_errors",
    "multicast_rcv_packets",
    "multicast_xmit_packets",
    "port_rcv_constraint_errors",
    "port_rcv_data",
    "port_rcv_errors",
    "port_rcv_packets",
    "port_rcv_remote_physical_errors",
    "port_rcv_switch_relay_errors",
    "port_xmit_constraint_errors",
    "port_xmit_data",
    "port_xmit_discards",
    "port_xmit_packets",
    "port_xmit_wait",
    "symbol_error",
    "unicast_rcv_packets",
    "unicast_xmit_packets"
];
const MLX_HW_COUNTERS: &'static [&'static str] = &[
    "duplicate_request",
    "implied_nak_seq_err",
    "lifespan",
    "local_ack_timeout_err",
    "np_cnp_sent",
    "np_ecn_marked_roce_packets",
    "out_of_buffer",
    "out_of_sequence",
    "packet_seq_err",
    "req_cqe_error",
    "req_cqe_flush_error",
    "req_remote_access_errors",
    "req_remote_invalid_request",
    "resp_cqe_error",
    "resp_cqe_flush_error",
    "resp_local_length_error",
    "resp_remote_access_errors",
    "rnr_nak_retry_err",
    "roce_adp_retrans",
    "roce_adp_retrans_to",
    "roce_slow_restart",
    "roce_slow_restart_cnps",
    "roce_slow_restart_trans",
    "rp_cnp_handled",
    "rp_cnp_ignored",
    "rx_atomic_requests",
    "rx_icrc_encapsulated",
    "rx_read_requests",
    "rx_write_requests",
];

const RXE_COUNTERS: &'static [&'static str] = &[
    "duplicate_request",
    "sent_pkts",
    "send_rnr_err",
    "send_err",
    "retry_rnr_exceeded_err",
    "retry_exceeded_err",
    "rdma_sends",
    "rdma_recvs",
    "rcvd_seq_err",
    "rcvd_rnr_err",
    "rcvd_pkts",
    "out_of_seq_request",
    "link_downed",
    "lifespan",
    "completer_retry_err",
    "ack_deferred",
];

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
    pub fn get_counters(&self, interface: &str, port: &str) -> Data {
        match self{
            Driver::Mlx => {

                let mlx_data = MlxData::default();

                for counter in MLX_COUNTERS{
                    let path = format!("/sys/class/infiniband/{}/ports/{}/counters/{}", interface, port, counter);
                    let counter_value = self.read_counter(&path);
                    set_field_by_name(&mlx_data, counter, counter_value); 
                }

                for counter in MLX_HW_COUNTERS{
                    let path = format!("/sys/class/infiniband/{}/ports/{}/hw_counters/{}", interface, port, counter);
                    let counter_value = self.read_counter(&path);
                    set_field_by_name(&mlx_data, counter, counter_value); 
                }

                let mut monitor_data = Data::default();
                monitor_data.data = Some(data::Data::Mlx(mlx_data));
                monitor_data

            },
            Driver::Rxe => {

                let rxe_data = RxeData::default();

                for counter in RXE_COUNTERS{
                    let path = format!("/sys/class/infiniband/{}/ports/{}/hw_counters/{}", interface, port, counter);
                    let counter_value = self.read_counter(&path);
                    set_field_by_name(&rxe_data, counter, counter_value); 
                }
                let mut monitor_data = Data::default();
                monitor_data.data = Some(data::Data::Rxe(rxe_data));
                monitor_data
            },

        }
    }
    fn read_counter(&self, path: &str) -> u64
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
}

fn set_field_by_name<T>(data: &T, field: &str, value: u64)
where
    T: serde::Serialize,
{
    let mut map = match serde_value::to_value(data){
        Ok(Value::Map(map)) => map,
        _ => {
            panic!("Error converting to value");
        }
    };


    let key = Value::String(field.to_string());
    let value = Value::U64(value);
    map.insert(key, value);
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
    //let mut history: HashMap<String,u64> = HashMap::new();
    info!("interfaces: {:?}", interfaces);
    loop{
        interval.tick().await;
        for interface in &interfaces{
            for port in &interface.ports{
                let c = driver.get_counters(&interface.name, port);
                
                /* 
                let hw_counters = HwCounters{
                    rx_write_requests: hw_counter_path(&interface.name, port, "rx_write_requests"),
                    rx_read_requests: hw_counter_path(&interface.name, port, "rx_read_requests"),
                    rx_atomic_requests: hw_counter_path(&interface.name, port, "rx_atomic_requests"),
                    resp_cqe_errors: hw_counter_path(&interface.name, port, "resp_cqe_error"),
                    req_cqe_errors: hw_counter_path(&interface.name, port, "req_cqe_error"),
                    resp_cqe_fl: hw_counter_path(&interface.name, port, "resp_cqe_flush_error"),
                    out_of_sequence: hw_counter_path(&interface.name, port, "out_of_sequence"),
                    out_of_buffer: hw_counter_path(&interface.name, port, "out_of_buffer"),
                    local_ack_timeout_errors: hw_counter_path(&interface.name, port, "local_ack_timeout_err"),
                    implied_nak_seq_errors: hw_counter_path(&interface.name, port, "implied_nak_seq_err"),
                    duplicate_request: hw_counter_path(&interface.name, port, "duplicate_request")
                };
                let (port_rcv_data, port_xmit_data, port_rcv_packets, port_xmit_packets) = if interface.name.contains("rxe"){
                    let port_rcv_data = linux_counter_path(&interface.linux_name, "rx_bytes");
                    let port_xmit_data = linux_counter_path(&interface.linux_name, "tx_bytes");
                    let port_rcv_packets = linux_counter_path(&interface.linux_name, "rx_packets");
                    let port_xmit_packets = linux_counter_path(&interface.linux_name, "tx_packets");
                    (port_rcv_data, port_xmit_data, port_rcv_packets, port_xmit_packets)
                } else {
                    let port_rcv_data = counter_path(&interface.name, port, "port_rcv_data") / 4;
                    let port_xmit_data = counter_path(&interface.name, port, "port_xmit_data") / 4;
                    let port_rcv_packets = counter_path(&interface.name, port, "port_rcv_packets");
                    let port_xmit_packets = counter_path(&interface.name, port, "port_xmit_packets");
                    (port_rcv_data, port_xmit_data, port_rcv_packets, port_xmit_packets)
                };

                let port_counters = PortCounters{
                    unicast_xmit_packets: counter_path(&interface.name, port, "unicast_xmit_packets"),
                    unicast_rcv_packets: counter_path(&interface.name, port, "unicast_rcv_packets"),
                    port_xmit_wait: counter_path(&interface.name, port, "port_xmit_wait"),
                    port_xmit_packets: port_xmit_packets,
                    port_xmit_data: port_xmit_data,
                    port_rcv_packets: port_rcv_packets,
                    port_rcv_errors: counter_path(&interface.name, port, "port_rcv_errors"),
                    port_rcv_data: port_rcv_data,
                    multicast_xmit_packets: counter_path(&interface.name, port, "multicast_xmit_packets"),
                    multicast_rcv_packets: counter_path(&interface.name, port, "multicast_rcv_packets"),
                };
                let mut per_sec_counter = PerSecCounters::default();

                let prev_port_rcv_packets_key = format!("{}_{}_rcv_packets", interface.name, port);
                if let Some(prev_port_rcv_packets) = history.get(&prev_port_rcv_packets_key){
                    if prev_port_rcv_packets.clone() < port_rcv_packets{
                        per_sec_counter.packets_recv = (port_rcv_packets as f64 - prev_port_rcv_packets.clone() as f64) / secs;
                    }
                }
                history.insert(prev_port_rcv_packets_key, port_rcv_packets);
                let prev_port_rcv_data_key = format!("{}_{}_rcv_data", interface.name, port);
                if let Some(prev_port_rcv_data) = history.get(&prev_port_rcv_data_key){
                    if prev_port_rcv_data.clone() < port_rcv_data{
                        per_sec_counter.bytes_recv = (port_rcv_data as f64 - prev_port_rcv_data.clone() as f64) / secs;
                    }
                }
                history.insert(prev_port_rcv_data_key, port_rcv_data);
                let prev_port_xmit_packets_key = format!("{}_{}_xmit_packets", interface.name, port);
                if let Some(prev_port_xmit_packets) = history.get(&prev_port_xmit_packets_key){
                    if prev_port_xmit_packets.clone() < port_xmit_packets{
                        per_sec_counter.packets_xmit = (port_xmit_packets as f64 - prev_port_xmit_packets.clone() as f64) / secs;
                    }
                }
                history.insert(prev_port_xmit_packets_key, port_xmit_packets);
                
                let prev_port_xmit_data_key = format!("{}_{}_xmit_data", interface.name, port);
                if let Some(prev_port_xmit_data) = history.get(&prev_port_xmit_data_key){
                    if prev_port_xmit_data.clone() < port_xmit_data{
                        per_sec_counter.bytes_xmit = (port_xmit_data as f64 - prev_port_xmit_data.clone() as f64) / secs;
                    }
                }
                history.insert(prev_port_xmit_data_key, port_xmit_data);
                


                let counters = Counters{
                    interface: interface.name.clone(),
                    port: port.to_string(),
                    port_counters,
                    hw_counters,
                    per_sec_counters: per_sec_counter,
                };
                if let Err(e) = tx.send(counters).await{
                    error!("Error sending counters: {}", e);
                }
                */
            }
        }
    }
}

fn counter_path(interface: &str, port: &str, counter_type: &str) -> u64
{
    let p = format!("/sys/class/infiniband/{}/ports/{}/counters/{}", interface, port, counter_type);
    let v = match std::fs::read_to_string(p){
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

fn linux_counter_path(interface: &str, counter_type: &str) -> u64
{
    let p = format!("/sys/class/net/{}/statistics/{}", interface, counter_type);
    let v = match std::fs::read_to_string(p){
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

fn hw_counter_path(interface: &str, port: &str, counter_type: &str) -> u64
{
    let p = format!("/sys/class/infiniband/{}/ports/{}/hw_counters/{}", interface, port, counter_type);
    let v = match std::fs::read_to_string(p){
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

