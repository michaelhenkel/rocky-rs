use std::{collections::HashMap, sync::Arc};

use crate::server::monitor::Stats;
use log::error;
use tokio::sync::{oneshot,mpsc, RwLock};

pub struct StatsDb{
    client: StatsDbClient,
    rx: Arc<RwLock<mpsc::Receiver<DbCommand>>>,
}


struct Db{
    stats: HashMap<String, InterfaceStats>,
}
impl Db{
    fn new() -> Self {
        Db{
            stats: HashMap::new(),
        }
    }
    fn insert(&mut self, stats: Stats) {
        let interface = self.stats.entry(stats.clone().meta.unwrap().interface.clone()).or_insert(InterfaceStats{
            port_stats: HashMap::new(),
        });
        interface.port_stats.insert(stats.clone().meta.unwrap().port.clone(), stats);
    }
    fn list(&self) -> Vec<Stats> {
        let mut stats = Vec::new();
        for (_address, interface) in &self.stats {
            for (_port, stat) in &interface.port_stats {
                stats.push(stat.clone());
            }
        }
        stats
    }
    fn get(&self, address: String, port: Option<String>) -> Option<Stats> {
        if let Some(interface) = self.stats.get(&address) {
            if let Some(port) = port {
                return interface.port_stats.get(&port).cloned();
            }
        }
        None
    }

}
struct InterfaceStats{
    port_stats: HashMap<String, Stats>,
}


impl StatsDb {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(100);
        StatsDb{
            client: StatsDbClient::new(tx),
            rx: Arc::new(RwLock::new(rx)),
        }
    }
    pub fn client(&self) -> StatsDbClient {
        self.client.clone()
    }
    pub async fn run(&self) -> anyhow::Result<()> {
        let mut db = Db::new();
        while let Some(db_command) = self.rx.write().await.recv().await {
            match db_command {
                DbCommand::Insert(stats) => {
                    db.insert(stats);
                },
                DbCommand::List{tx} => {
                    let stats = db.list();
                    if let Err(_e) = tx.send(stats){
                        error!("Error sending stats");
                    }
                },
                DbCommand::Get{address, port, tx} => {
                    let stats = db.get(address, port);
                    if let Err(_e) = tx.send(stats){
                        error!("Error sending stats");
                    }
                }
            }
        }
        Ok(())
    }

}

#[derive(Clone)]
pub struct StatsDbClient{
    tx: mpsc::Sender<DbCommand>,
}

impl StatsDbClient {
    pub fn new(tx: mpsc::Sender<DbCommand>) -> Self {
        StatsDbClient{tx}
    }
    pub async fn insert(&self, stats: Stats) -> anyhow::Result<()> {
        self.tx.send(DbCommand::Insert(stats)).await?;
        Ok(())
    }
    pub async fn list(&self) -> anyhow::Result<Vec<Stats>> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(DbCommand::List{tx}).await?;
        Ok(rx.await?)
    }
    pub async fn get(&self, address: String, port: Option<String>) -> anyhow::Result<Option<Stats>> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(DbCommand::Get{address, port, tx}).await?;
        Ok(rx.await?)
    }
}

pub enum DbCommand{
    Insert(Stats),
    List{
        tx: oneshot::Sender<Vec<Stats>>,
    },
    Get{
        address: String,
        port: Option<String>,
        tx: oneshot::Sender<Option<Stats>>,
    }
}