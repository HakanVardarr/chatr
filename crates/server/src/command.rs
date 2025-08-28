use super::response::Response;
use std::net::SocketAddr;
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum Command {
    Hello {
        username: String,
        addr: SocketAddr,
        respond_to: oneshot::Sender<Response>,
        private_sender: mpsc::Sender<Response>,
    },
    Message {
        from: String,
        body: String,
        respond_to: oneshot::Sender<Response>,
    },
    Quit {
        username: String,
    },
    PrivateMessage {
        from: String,
        to: String,
        body: String,
        respond_to: oneshot::Sender<Response>,
    },
}
