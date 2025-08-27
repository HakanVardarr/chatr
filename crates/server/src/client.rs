use crate::MAX_USER_SIZE;

use super::command::Command;
use super::response::Response;
use std::net::SocketAddr;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
    sync::mpsc,
};

#[derive(Debug)]
pub struct User {
    pub name: String,
    pub _addr: SocketAddr,
    pub private_sender: mpsc::Sender<Response>,
}

fn is_username_valid(username: &str) -> bool {
    !username.is_empty() && username.chars().all(|c| c.is_ascii_alphabetic())
}

pub async fn handle_client(stream: TcpStream, sender: mpsc::Sender<Command>) -> anyhow::Result<()> {
    let (reader_half, mut w) = stream.into_split();

    let mut reader = BufReader::new(reader_half);

    let mut validated = false;

    let mut _username = String::new();
    let (private_tx, mut private_rx) = mpsc::channel(MAX_USER_SIZE);

    loop {
        let mut line = String::new();

        tokio::select! {
            read_res = reader.read_line(&mut line) => {
                let n = read_res?;
                if n == 0 {
                    if validated {
                        let _ = sender.send(Command::Quit { username: _username.clone() }).await;
                    }
                    return Ok(());
                }
                let input = line.trim().to_string();
                line.clear();
                if let Some((command, payload)) = input.split_once("|") {
                    let command = command.trim();
                    let payload = payload.trim();

                    match command {
                        "HELLO" => {
                            if validated {
                                w.write_all(b"ERROR | 04 | You are already validated.\n").await?;
                            } else if is_username_valid(payload) {
                                let (resp_tx, resp_rx) = oneshot::channel();

                                sender
                                .send(Command::Hello {
                                        username: payload.into(),
                                        addr: w.peer_addr().unwrap(),
                                        respond_to: resp_tx,
                                        private_sender: private_tx.clone(),
                                }).await?;

                                match resp_rx.await {
                                    Ok(Response::Welcome { username, user_count }) => {
                                        validated = true;
                                        let msg = format!("WELCOME | {username} | {user_count}\n");
                                        w.write_all(msg.as_bytes()).await?;
                                        _username = username
                                    },
                                    Ok(Response::Error { error_code, error_message }) => {
                                        let msg = format!("ERROR | {error_code:02} | {error_message}\n");
                                        w.write_all(msg.as_bytes()).await?;
                                    },
                                    _  => {},
                                }
                            } else {
                                w.write_all(b"ERROR | 03 | Invalid username.\n").await?;
                            }
                        },
                        "MESSAGE" => {
                            if !validated {
                                let msg = "ERROR | 06 | Please validate yourself.\n";
                                w.write_all(msg.as_bytes()).await?;
                            }

                            let _ = sender.send(Command::Message { from: _username.clone(), body: payload.into() }).await;
                        }
                        "QUIT" => {
                            if validated {
                                let _ = sender.send(Command::Quit { username: _username.clone() }).await;
                                return Ok(());
                            } else {
                                let msg = "ERROR | 06 | Please validate yourself.\n";
                                w.write_all(msg.as_bytes()).await?;
                            }
                        }
                        "PRIVATE" => {
                            if let Some((to, body)) = payload.split_once("|") && validated {
                                let to = to.trim();
                                let body = body.trim();

                                let (resp_tx, resp_rx) = oneshot::channel();

                                let _ = sender.send(Command::PrivateMessage { from: _username.clone(), to: to.into(), body: body.into(), respond_to: resp_tx}).await;

                                match resp_rx.await {
                                    Ok(Response::Success) => {},
                                    Ok(Response::Error { error_code, error_message }) => {
                                        let msg = format!("ERROR | {error_code:02} | {error_message}\n");
                                        w.write_all(msg.as_bytes()).await?;
                                    },
                                    _  => {},
                                }
                            } else {
                                w.write_all(b"ERROR | 01 | Please follow protocol.\n").await?;
                            }
                        }
                        _ => {
                            let msg = format!("ERROR | 02 | Invalid command '{}'.\n", command);
                            w.write_all(msg.as_bytes()).await?;
                        },
                    }
                } else {
                    w.write_all(b"ERROR | 01 | Please follow protocol.\n").await?;
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
                    if validated && username == _username { continue; }
                    let out = format!("LEFT | {}\n", username);
                    w.write_all(out.as_bytes()).await?;
                }
            }
        }
    }
}
