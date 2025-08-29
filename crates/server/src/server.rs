use super::client::User;
use crate::protocol::request::Request;
use std::collections::HashMap;
use tokio::sync::mpsc;

pub struct Server {
    pub connections: HashMap<String, User>,
    reciever: mpsc::Receiver<Request>,
}

impl Server {
    pub fn new(reciever: mpsc::Receiver<Request>) -> Self {
        Self {
            connections: HashMap::new(),
            reciever,
        }
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        while let Some(req) = self.reciever.recv().await {
            let _ = req.process(self).await;
        }

        Ok(())
    }
}
