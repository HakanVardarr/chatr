use super::client::User;
use super::command::Command;
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
                    private_sender,
                });

                respond_to
                    .send(Response::Welcome {
                        username,
                        user_count: connections.len() as u32,
                    })
                    .unwrap();
            }
            Command::Message { from, body } => {
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
            }
            Command::Quit { username } => {
                if let Some(pos) = connections.iter().position(|user| user.name == username) {
                    connections.remove(pos);
                    for connection in &connections {
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
                if let Some(user) = connections.iter().find(|user| user.name == to) {
                    let _ = user
                        .private_sender
                        .send(Response::Chat {
                            from,
                            body,
                            is_private: true,
                        })
                        .await;

                    let _ = respond_to.send(Response::Success);
                } else if from == to {
                    let _ = respond_to.send(Response::Error {
                        error_code: 7,
                        error_message: "You cannot send a private message to yourself.".into(),
                    });
                } else {
                    let _ = respond_to.send(Response::Error {
                        error_code: 8,
                        error_message: "User does not exists.".into(),
                    });
                }
            }
        }
    }
}
