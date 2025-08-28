use super::client::User;
use super::command::Command;
use super::error::ProtocolError;
use super::response::Response;
use tokio::sync::mpsc;

pub async fn run_server(mut rx: mpsc::Receiver<Command>) {
    let mut connections = vec![];

    while let Some(command) = rx.recv().await {
        match command {
            Command::Hello {
                username,
                addr,
                respond_to,
                private_sender,
            } => {
                if connections.iter().any(|conn: &User| conn.name == username) {
                    let _ = respond_to.send(Response::Error(ProtocolError::UserExists));
                    continue;
                }

                connections.push(User {
                    name: username.clone(),
                    _addr: addr,
                    private_sender,
                });

                let _ = respond_to.send(Response::Welcome {
                    username: username.clone(),
                    user_count: connections.len() as u32,
                });

                for connection in &connections {
                    if connection.name != username.clone() {
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
                for connection in &connections {
                    if connection.name == from {
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
                if let Some(pos) = connections.iter().position(|user| user.name == username) {
                    connections.remove(pos);
                    for connection in &connections {
                        if connection.name == username {
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
                } else if let Some(user) = connections.iter().find(|user| user.name == to) {
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
