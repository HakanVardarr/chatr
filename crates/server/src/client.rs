use crate::protocol::request::Request;

use super::MAX_USER_SIZE;
use super::protocol::command::Command;
use super::protocol::error::ProtocolError;
use super::protocol::response::Response;
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, AsyncWriteExt, BufReader},
    sync::mpsc,
};

#[derive(Debug)]
pub struct User {
    #[allow(dead_code)]
    pub name: String,
    pub private_sender: mpsc::Sender<Response>,
}

#[derive(Debug)]
pub struct ClientState {
    pub validated: bool,
    pub username: String,
}

impl ClientState {
    fn is_validated(&self) -> bool {
        self.validated
    }

    async fn write_if_not_validated<W>(&self, w: &mut W) -> anyhow::Result<()>
    where
        W: AsyncWriteExt + Unpin,
    {
        if !self.validated {
            let resp = Response::Error(ProtocolError::NotValidated);
            w.write_all(format!("{resp}\n").as_bytes()).await?;
        }

        Ok(())
    }

    fn validate(&mut self) {
        self.validated = true;
    }

    fn is_username_valid(&self) -> bool {
        !self.username.is_empty() && self.username.chars().all(|c| c.is_ascii_alphabetic())
    }
}

async fn handle_read_line<W>(
    bytes_read: usize,
    line: &mut String,
    state: &mut ClientState,
    sender: &mpsc::Sender<Request>,
    w: &mut W,
    private_sender: mpsc::Sender<Response>,
) -> anyhow::Result<()>
where
    W: AsyncWriteExt + Unpin,
{
    if bytes_read == 0 {
        if state.is_validated() {
            let (req, _) = Request::new(Command::Quit {
                username: state.username.clone(),
            });

            let _ = sender.send(req).await;
        }
        return Ok(());
    }

    let input = line.trim().to_string();
    line.clear();

    if let Some((command, body)) = input.split_once("|") {
        let command = command.trim();
        let body = body.trim();

        match command {
            "HELLO" => {
                if state.is_validated() {
                    let resp = Response::Error(ProtocolError::AlreadyValidated);
                    w.write_all(format!("{resp}\n").as_bytes()).await?;
                    return Ok(());
                }

                state.username = body.into();
                if !state.is_username_valid() {
                    let resp = Response::Error(ProtocolError::InvalidUsername);
                    w.write_all(format!("{resp}\n").as_bytes()).await?;
                    return Ok(());
                }

                let (req, rx) = Request::new(Command::Hello {
                    username: body.into(),
                    private_sender,
                });

                sender.send(req).await?;

                let response = rx.await?;
                match &response {
                    Response::Welcome {
                        username: _,
                        user_count: _,
                    } => {
                        state.validate();
                        w.write_all(format!("{response}\n").as_bytes()).await?;
                    }
                    Response::Error(_) => {
                        w.write_all(format!("{response}\n").as_bytes()).await?;
                    }
                    _ => {}
                }
            }
            "MESSAGE" => {
                state.write_if_not_validated(w).await?;
                if state.is_validated() {
                    let (req, rx) = Request::new(Command::Message {
                        from: state.username.clone(),
                        body: body.into(),
                    });

                    sender.send(req).await?;

                    let response = rx.await?;
                    match &response {
                        Response::Success => {
                            w.write_all(format!("{response}\n").as_bytes()).await?;
                        }
                        Response::Error(_) => {
                            w.write_all(format!("{response}\n").as_bytes()).await?;
                        }
                        _ => {}
                    }
                }
            }
            "QUIT" => {
                if state.is_validated() {
                    let (req, _) = Request::new(Command::Quit {
                        username: state.username.clone(),
                    });

                    sender.send(req).await?;
                    w.shutdown().await?;
                    return Ok(());
                }
            }
            "PRIVATE" => {
                state.write_if_not_validated(w).await?;
                if let Some((to, body)) = body.split_once("|")
                    && state.is_validated()
                {
                    let to = to.trim();
                    let body = body.trim();

                    let (req, rx) = Request::new(Command::PrivateMessage {
                        from: state.username.clone(),
                        to: to.into(),
                        body: body.into(),
                    });
                    let _ = sender.send(req).await;

                    let response = rx.await?;
                    match &response {
                        Response::Success => {
                            w.write_all(format!("{response}\n").as_bytes()).await?;
                        }
                        Response::Error(_) => {
                            w.write_all(format!("{response}\n").as_bytes()).await?;
                        }
                        _ => {}
                    }
                }
            }
            _ => {
                let resp = Response::Error(ProtocolError::InvalidCommand);
                w.write_all(format!("{resp} {command}\n").as_bytes())
                    .await?;
            }
        }
    } else {
        let resp = Response::Error(ProtocolError::InvalidFormat);
        w.write_all(format!("{resp}\n").as_bytes()).await?;
    }

    Ok(())
}

pub async fn handle_client<S>(stream: S, sender: mpsc::Sender<Request>) -> anyhow::Result<()>
where
    S: AsyncRead + AsyncWriteExt + Unpin + Send + 'static,
{
    let (reader_half, mut w) = tokio::io::split(stream);
    let mut reader = BufReader::new(reader_half);

    let mut state = ClientState {
        validated: false,
        username: String::new(),
    };
    let (private_tx, mut private_rx) = mpsc::channel(MAX_USER_SIZE);

    let mut line = String::new();
    loop {
        tokio::select! {
            bytes_read = reader.read_line(&mut line) => {
                match bytes_read {
                    Ok(0) => {
                        if state.is_validated() {
                            let (req, _) = Request::new(Command::Quit {
                                username: state.username.clone(),
                            });

                            let _ = sender.send(req).await;
                        }
                        return Ok(());
                    }
                    Ok(n) => {handle_read_line(n, &mut line, &mut state, &sender, &mut w, private_tx.clone()).await?;}
                    Err(_) => {
                        if state.is_validated() {
                            let (req, _) = Request::new(Command::Quit {
                                username: state.username.clone(),
                            });

                            let _ = sender.send(req).await;
                        }
                        return Ok(());
                    }
                }
            }

            read = private_rx.recv() => {
                if let Some(Response::Chat { from, body, is_private }) = read {
                    let c_type = if is_private {
                        "PRIVATE"
                    } else {
                        "CHAT"
                    };

                    let msg = format!("{} | {from} | {body}\n", c_type);
                    w.write_all(msg.as_bytes()).await?;
                } else if let Some(Response::Quit {username}) = read {
                    if state.is_validated() && username == state.username { continue; }
                    let out = format!("LEFT | {}\n", username);
                    w.write_all(out.as_bytes()).await?;
                } else if let Some(Response::Join {username}) = read {
                    if state.is_validated() && username == state.username { continue; }
                    let out = format!("JOIN | {}\n", username);
                    w.write_all(out.as_bytes()).await?;
                }
            }
        }
    }
}
