use crate::client::handle_client;
use crate::command::Command;
use crate::server::run_server;

use tokio::net::TcpListener;
use tokio::sync::mpsc;

mod client;
mod command;
mod response;
mod server;

const ADDR: &str = "0.0.0.0:3030";
const MAX_USER_SIZE: usize = 50;

pub async fn run() -> anyhow::Result<()> {
    let listener = TcpListener::bind(ADDR).await?;

    let (tx, rx) = mpsc::channel::<Command>(MAX_USER_SIZE);

    tokio::spawn(run_server(rx));

    loop {
        let (socket, _) = listener.accept().await?;
        let sender = tx.clone();

        tokio::spawn(async move {
            let _ = handle_client(socket, sender).await;
        });

        std::thread::yield_now();
    }
}
