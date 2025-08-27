use super::response::Response;
use std::net::SocketAddr;

#[derive(Debug)]
pub enum Command {
    Hello {
        username: String,
        addr: SocketAddr,
        respond_to: oneshot::Sender<Response>,
    },
    Message {
        from: String,
        body: String,
    },
    Quit {
        username: String,
    },
}
