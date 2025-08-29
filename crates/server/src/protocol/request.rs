use super::command::Command;
use super::error::ProtocolError;
use super::response::Response;
use crate::client::User;
use crate::server::Server;

pub struct Request {
    pub cmd: Command,
    pub respond_to: oneshot::Sender<Response>,
}

impl Request {
    pub fn new(cmd: Command) -> (Self, oneshot::Receiver<Response>) {
        let (tx, rx) = oneshot::channel();
        (
            Self {
                cmd,
                respond_to: tx,
            },
            rx,
        )
    }

    pub async fn process(self, server: &mut Server) -> anyhow::Result<()> {
        match self.cmd {
            Command::Hello {
                username,
                private_sender,
            } => {
                if server.connections.contains_key(&username) {
                    self.respond_to
                        .send(Response::Error(ProtocolError::UserExists))?;
                    return Ok(());
                }
                server.connections.insert(
                    username.clone(),
                    User {
                        name: username.clone(),
                        private_sender,
                    },
                );

                self.respond_to.send(Response::Welcome {
                    username: username.clone(),
                    user_count: server.connections.len() as u32,
                })?;

                for (name, connection) in &server.connections {
                    if name != &username {
                        connection
                            .private_sender
                            .send(Response::Join {
                                username: username.clone(),
                            })
                            .await?;
                    }
                }
            }
            Command::Message { from, body } => {
                for (name, connection) in &server.connections {
                    if name == &from {
                        continue;
                    }

                    connection
                        .private_sender
                        .send(Response::Chat {
                            from: from.clone(),
                            body: body.clone(),
                            is_private: false,
                        })
                        .await?;
                }
                self.respond_to.send(Response::Success)?;
            }
            Command::Quit { username } => {
                if server.connections.remove(&username).is_some() {
                    for connection in server.connections.values() {
                        connection
                            .private_sender
                            .send(Response::Quit {
                                username: username.clone(),
                            })
                            .await?;
                    }
                }
            }
            Command::PrivateMessage { from, to, body } => {
                if from == to {
                    self.respond_to
                        .send(Response::Error(ProtocolError::MessageYourself))?;
                } else if let Some(user) = server.connections.get(&to) {
                    user.private_sender
                        .send(Response::Chat {
                            from,
                            body,
                            is_private: true,
                        })
                        .await?;
                    self.respond_to.send(Response::Success)?;
                } else {
                    self.respond_to
                        .send(Response::Error(ProtocolError::UserDoesntExists))?;
                }
            }
        }

        Ok(())
    }
}
