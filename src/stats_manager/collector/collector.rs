use std::{collections::HashMap, sync::Arc};
use log::{error, info};
use tokio::sync::RwLock;
use crate::connection_manager::connection_manager::{
    CounterFilter, InterfaceCounter, PortCounter, HwCounter, MonitorCounters
};
use crate::monitor_client::monitor_client::Client;
use monitor::server::monitor::{Stats as MonitorStats, Meta as MonitorMeta, Data as MonitorData};
use gethostname;

pub struct Collector{
    interfaces: Vec<Interface>,
    client: CollectorClient,
    rx: Arc<RwLock<tokio::sync::mpsc::Receiver<CollectorCommand>>>,
    freq: u64,
    monitor_client: Option<Client>,
}

#[derive(Clone)]
struct Interface{
    name: String,
    ports: Vec<String>,
}

#[derive(Clone)]
struct HwCounters{
    rx_write_requests: u64,
    rx_read_requests: u64,
    rx_atomic_requests: u64,
    resp_cqe_errors: u64,
    req_cqe_errors: u64,
    resp_cqe_fl: u64,
    out_of_sequence: u64,
    out_of_buffer: u64,
    local_ack_timeout_errors: u64,
    implied_nak_seq_errors: u64,
    duplicate_request: u64,
}

impl Into<HwCounter> for HwCounters{
    fn into(self) -> HwCounter{
        HwCounter{
            rx_write_requests: self.rx_write_requests,
            rx_read_requests: self.rx_read_requests,
            rx_atomic_requests: self.rx_atomic_requests,
            resp_cqe_errors: self.resp_cqe_errors,
            req_cqe_errors: self.req_cqe_errors,
            resp_cqe_fl: self.resp_cqe_fl,
            out_of_sequence: self.out_of_sequence,
            out_of_buffer: self.out_of_buffer,
            local_ack_timeout_errors: self.local_ack_timeout_errors,
            implied_nak_seq_errors: self.implied_nak_seq_errors,
            duplicate_request: self.duplicate_request,
        }
    }
}

#[derive(Clone)]
struct PortCounters{
    unicast_xmit_packets: u64,
    unicast_rcv_packets: u64,
    port_xmit_wait: u64,
    port_xmit_packets: u64,
    port_xmit_data: u64,
    port_rcv_packets: u64,
    port_rcv_errors: u64,
    port_rcv_data: u64,
    multicast_xmit_packets: u64,
    multicast_rcv_packets: u64,
}

impl Into<PortCounter> for PortCounters{
    fn into(self) -> PortCounter{
        PortCounter{
            unicast_xmit_packets: self.unicast_xmit_packets,
            unicast_rcv_packets: self.unicast_rcv_packets,
            port_xmit_wait: self.port_xmit_wait,
            port_xmit_packets: self.port_xmit_packets,
            port_xmit_data: self.port_xmit_data,
            port_rcv_packets: self.port_rcv_packets,
            port_rcv_errors: self.port_rcv_errors,
            port_rcv_data: self.port_rcv_data,
            multicast_xmit_packets: self.multicast_xmit_packets,
            multicast_rcv_packets: self.multicast_rcv_packets,
        }
    }
}

#[derive(Clone, Default)]
struct PerSecCounters{
    bytes_recv: f64,
    packets_recv: f64,
    bytes_xmit: f64,
    packets_xmit: f64,
}

struct Counters{
    interface: String,
    port: String,
    port_counters: PortCounters,
    hw_counters: HwCounters,
    per_sec_counters: PerSecCounters,
}

impl Interface{
    pub fn new(name: String) -> Interface{
        let ports = std::fs::read_dir(format!("/sys/class/infiniband/{}/ports", name)).unwrap()
            .map(|entry| entry.unwrap().file_name().into_string().unwrap())
            .collect();
        Interface{
            name,
            ports,
        }
    }
}

impl Collector{
    pub fn new(freq: u64, monitor_client: Option<Client>) -> Collector{
        let interface_names: Vec<String> = std::fs::read_dir("/sys/class/infiniband").unwrap()
            .map(|entry| entry.unwrap().file_name().into_string().unwrap())
            .collect();
        let mut interfaces = Vec::new();
        for name in interface_names{
            interfaces.push(Interface::new(name));
        }
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        Collector{
            interfaces,
            client: CollectorClient::new(tx),
            rx: Arc::new(RwLock::new(rx)),
            freq,
            monitor_client,
        }
    }
    pub fn client(&self) -> CollectorClient{
        self.client.clone()
    }
    pub async fn run(&self) -> anyhow::Result<()>{
        let interfaces = self.interfaces.clone();
        let mut jh_list = Vec::new();
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        let freq = self.freq;
        let jh = tokio::spawn(async move{
            if let Err(e) = count(freq, interfaces, tx).await{
                error!("Error counting: {}", e);
            }
        });
        jh_list.push(jh);
        let hostname = gethostname::gethostname().to_string_lossy().to_string();
        let monitor_client = self.monitor_client.clone();
        let command_rx = self.rx.clone();
        let jh = tokio::spawn(async move{
            let mut command_rx = command_rx.write().await;
            let mut receiver_list: HashMap<String,(tokio::sync::mpsc::Sender<InterfaceCounter>, CounterFilter)> = HashMap::new();
            loop{
                tokio::select! {
                    counter = rx.recv() => {
                        if let Some(counter) = counter{
                            for (_, (receiver_tx, counter_filter)) in &receiver_list{
                                if let Some(interface_filter) = &counter_filter.interface{
                                    if interface_filter.clone() != counter.interface{
                                        continue;
                                    }
                                }
                                if let Some(port_filter) = &counter_filter.port{
                                    if port_filter.clone() != counter.port.parse::<u32>().unwrap(){
                                        continue;
                                    }
                                }
                            
                                let grpc_hw_counter: HwCounter = counter.hw_counters.clone().into();
                                let grpc_port_counter: PortCounter = counter.port_counters.clone().into();
                                let grpc_counters = MonitorCounters{
                                    hw_counter: Some(grpc_hw_counter),
                                    port_counter: Some(grpc_port_counter),
                                };
                                let interface_counter = InterfaceCounter{
                                    name: counter.interface.clone(),
                                    counters: HashMap::from([(counter.port.clone().parse::<u32>().unwrap(), grpc_counters)]),
                                };
                                receiver_tx.send(interface_counter).await.unwrap();
                            }
                            if let Some(monitor_client) = monitor_client.clone(){
                                let monitor_data = MonitorData{
                                    unicast_xmit_packets: counter.port_counters.unicast_xmit_packets,
                                    unicast_rcv_packets: counter.port_counters.unicast_rcv_packets,
                                    port_xmit_wait: counter.port_counters.port_xmit_wait,
                                    port_xmit_packets: counter.port_counters.port_xmit_packets,
                                    port_xmit_data: counter.port_counters.port_xmit_data,
                                    port_rcv_packets: counter.port_counters.port_rcv_packets,
                                    port_rcv_errors: counter.port_counters.port_rcv_errors,
                                    port_rcv_data: counter.port_counters.port_rcv_data,
                                    multicast_xmit_packets: counter.port_counters.multicast_xmit_packets,
                                    multicast_rcv_packets: counter.port_counters.multicast_rcv_packets,
                                    rx_write_requests: counter.hw_counters.rx_write_requests,
                                    rx_read_requests: counter.hw_counters.rx_read_requests,
                                    rx_atomic_requests: counter.hw_counters.rx_atomic_requests,
                                    resp_cqe_errors: counter.hw_counters.resp_cqe_errors,
                                    req_cqe_errors: counter.hw_counters.req_cqe_errors,
                                    resp_cqe_fl: counter.hw_counters.resp_cqe_fl,
                                    out_of_sequence: counter.hw_counters.out_of_sequence,
                                    out_of_buffer: counter.hw_counters.out_of_buffer,
                                    local_ack_timeout_errors: counter.hw_counters.local_ack_timeout_errors,
                                    implied_nak_seq_errors: counter.hw_counters.implied_nak_seq_errors,
                                    duplicate_request: counter.hw_counters.duplicate_request,
                                    bytes_xmit_per_sec: counter.per_sec_counters.bytes_xmit,
                                    packets_xmit_per_sec: counter.per_sec_counters.packets_xmit,
                                    bytes_rcv_per_sec: counter.per_sec_counters.bytes_recv,
                                    packets_rcv_per_sec: counter.per_sec_counters.packets_recv,
                                };
                                let stats = MonitorStats{
                                    meta: Some(MonitorMeta{
                                        hostname: hostname.clone(),
                                        interface: counter.interface.clone(),
                                        port: counter.port.clone(),
                                    }),
                                    data: Some(monitor_data),
                                };
                                if let Err(e) = monitor_client.send(stats).await{
                                    error!("Error sending stats: {}", e);
                                }
                                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                            }
                        }
                    },
                    command = command_rx.recv() => {
                        if let Some(command) = command{
                            match command{
                                CollectorCommand::Register{id, tx, counter_filter} => {
                                    info!("adding monitor for id: {} to map", id);
                                    receiver_list.insert(id, (tx, counter_filter));
                                }
                                CollectorCommand::UnRegister{id} => {
                                    receiver_list.remove(&id);
                                }
                            }
                        }
                    },
                }
            }
        });
        jh_list.push(jh);
        futures::future::join_all(jh_list).await;
        Ok(())
    }
}

#[derive(Clone)]
pub struct CollectorClient{
    tx: tokio::sync::mpsc::Sender<CollectorCommand>,
}

impl CollectorClient{
    fn new(tx: tokio::sync::mpsc::Sender<CollectorCommand>) -> CollectorClient{
        CollectorClient{tx}
    }
    pub async fn register(&self, id: String, tx: tokio::sync::mpsc::Sender<InterfaceCounter>, counter_filter: CounterFilter) -> anyhow::Result<()>{
        info!("Registering counter monitor for id: {}", id);
        self.tx.send(CollectorCommand::Register{id, tx, counter_filter}).await?;
        Ok(())
    }
    pub async fn unregister(&self, id: String) -> anyhow::Result<()>{
        self.tx.send(CollectorCommand::UnRegister{id}).await?;
        Ok(())
    }
}

enum CollectorCommand{
    Register{
        id: String,
        tx: tokio::sync::mpsc::Sender<InterfaceCounter>,
        counter_filter: CounterFilter,
    },
    UnRegister{
        id: String,
    }
}

async fn count(freq: u64, interfaces: Vec<Interface>, tx: tokio::sync::mpsc::Sender<Counters>) -> anyhow::Result<()>{
    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(freq));
    let secs: f64 = freq as f64 / 1000 as f64;
    let mut history: HashMap<String,u64> = HashMap::new();
    loop{
        interval.tick().await;
        for interface in &interfaces{
            for port in &interface.ports{
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
                let port_rcv_data = counter_path(&interface.name, port, "port_rcv_data") / 4;
                let port_xmit_data = counter_path(&interface.name, port, "port_xmit_data") / 4;
                let port_rcv_packets = counter_path(&interface.name, port, "port_rcv_packets");
                let port_xmit_packets = counter_path(&interface.name, port, "port_xmit_packets");
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
                    port: port.clone(),
                    port_counters,
                    hw_counters,
                    per_sec_counters: per_sec_counter,
                };
                if let Err(e) = tx.send(counters).await{
                    error!("Error sending counters: {}", e);
                }
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

