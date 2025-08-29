use super::response::Response;
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum Command {
    Hello {
        username: String,
        private_sender: mpsc::Sender<Response>,
    },
    Message {
        from: String,
        body: String,
    },
    Quit {
        username: String,
    },
    PrivateMessage {
        from: String,
        to: String,
        body: String,
    },
}
