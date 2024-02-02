use std::collections::VecDeque;

use log::info;

use crate::initiator::{initiator, listener::listener::SendRequest};

pub struct Queue{}

impl Queue {
    pub fn new() -> Queue {
        Queue{}
    }
    pub async fn run(&mut self, mut rx: tokio::sync::mpsc::Receiver<SendRequest>) -> anyhow::Result<()> {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(10));
        let mut queue = VecDeque::new();
        loop{
            tokio::select! {
                _ = interval.tick() => {
                    if let Some(request) = queue.pop_front() {
                        info!("sending request: {:?}", request);
                        initiator::initiate(request).await?;
                    }
                },
                request = rx.recv() => {
                    if let Some(request) = request {
                        info!("received request: {:?}", request);
                        queue.push_back(request);
                    }
                },
            }
        }
    }
}