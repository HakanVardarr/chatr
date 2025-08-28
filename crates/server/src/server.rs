use super::client::User;
use super::command::Command;
use super::error::ProtocolError;
use super::response::Response;
use tokio::sync::mpsc;

use std::collections::HashMap;

pub async fn run_server(mut rx: mpsc::Receiver<Command>) {
    let mut connections: HashMap<String, User> = HashMap::new();

    while let Some(command) = rx.recv().await {
        match command {
            Command::Hello {
                username,
                respond_to,
                private_sender,
            } => {
                if connections.contains_key(&username) {
                    let _ = respond_to.send(Response::Error(ProtocolError::UserExists));
                    continue;
                }

                connections.insert(
                    username.clone(),
                    User {
                        name: username.clone(),
                        private_sender,
                    },
                );

                let _ = respond_to.send(Response::Welcome {
                    username: username.clone(),
                    user_count: connections.len() as u32,
                });

                for (name, connection) in &connections {
                    if name != &username {
                        let _ = connection
                            .private_sender
                            .send(Response::Join {
                                username: username.clone(),
                            })
                            .await;
                    }
                }
            }
            Command::Message {
                from,
                body,
                respond_to,
            } => {
                for (name, connection) in &connections {
                    if name == &from {
                        continue;
                    }
                    let _ = connection
                        .private_sender
                        .send(Response::Chat {
                            from: from.clone(),
                            body: body.clone(),
                            is_private: false,
                        })
                        .await;
                }
                let _ = respond_to.send(Response::Success);
            }
            Command::Quit { username } => {
                if connections.remove(&username).is_some() {
                    for (name, connection) in &connections {
                        if name == &username {
                            continue;
                        }
                        let _ = connection
                            .private_sender
                            .send(Response::Quit {
                                username: username.clone(),
                            })
                            .await;
                    }
                }
            }
            Command::PrivateMessage {
                from,
                to,
                body,
                respond_to,
            } => {
                if from == to {
                    let _ = respond_to.send(Response::Error(ProtocolError::MessageYourself));
                } else if let Some(user) = connections.get(&to) {
                    let _ = user
                        .private_sender
                        .send(Response::Chat {
                            from,
                            body,
                            is_private: true,
                        })
                        .await;
                    let _ = respond_to.send(Response::Success);
                } else {
                    let _ = respond_to.send(Response::Error(ProtocolError::UserDoesntExists));
                }
            }
        }
    }
}
