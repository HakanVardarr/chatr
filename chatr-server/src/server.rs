use super::client::User;
use super::command::Command;
use super::response::Response;
use tokio::sync::{broadcast, mpsc};

pub async fn run_server(mut rx: mpsc::Receiver<Command>, sender: broadcast::Sender<Response>) {
    let mut connections = vec![];

    while let Some(command) = rx.recv().await {
        match command {
            Command::Hello {
                username,
                addr,
                respond_to,
            } => {
                if connections.iter().any(|conn: &User| conn.name == username) {
                    respond_to
                        .send(Response::Error {
                            error_code: 5,
                            error_message: "User already exists.".into(),
                        })
                        .unwrap();
                    continue;
                }

                connections.push(User {
                    name: username.clone(),
                    _addr: addr,
                });

                respond_to
                    .send(Response::Welcome {
                        username,
                        user_count: connections.len() as u32,
                    })
                    .unwrap();
            }
            Command::Message { from, body } => {
                let _ = sender.send(Response::Chat { from, body });
            }
            Command::Quit { username } => {
                if let Some(pos) = connections.iter().position(|user| user.name == username) {
                    connections.remove(pos);
                    let _ = sender.send(Response::Quit { username });
                }
            }
        }
    }
}
