use crate::client::handle_client;
use crate::command::Command;
use crate::response::Response;
use crate::server::run_server;
use tokio::net::TcpListener;
use tokio::sync::{broadcast, mpsc};

mod client;
mod command;
mod response;
mod server;

const ADDR: &str = "127.0.0.1:3030";
const MAX_USER_SIZE: usize = 50;

pub async fn run() -> anyhow::Result<()> {
    let listener = TcpListener::bind(ADDR).await?;

    let (tx, rx) = mpsc::channel::<Command>(MAX_USER_SIZE);
    let (btx, _) = broadcast::channel::<Response>(MAX_USER_SIZE);

    let sender = btx.clone();
    tokio::spawn(run_server(rx, sender));

    loop {
        let (socket, _) = listener.accept().await?;
        let sender = tx.clone();
        let reciever = btx.subscribe();

        tokio::spawn(async move {
            let _ = handle_client(socket, sender, reciever).await;
        });

        std::thread::yield_now();
    }
}
