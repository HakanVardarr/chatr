use crate::error::ProtocolError;
use core::fmt;

#[derive(Clone, Debug)]
pub enum Response {
    Success,
    Error(ProtocolError),
    Welcome {
        username: String,
        user_count: u32,
    },
    Chat {
        from: String,
        body: String,
        is_private: bool,
    },
    Quit {
        username: String,
    },
    Join {
        username: String,
    },
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Response::Error(err) => write!(f, "ERROR | {:02} | {}", err.code(), err.message()),
            Response::Welcome {
                username,
                user_count,
            } => {
                write!(f, "WELCOME | {username} | {user_count}")
            }
            Response::Chat {
                from,
                body,
                is_private,
            } => {
                if *is_private {
                    write!(f, "PRIVATE | {from} | {body}")
                } else {
                    write!(f, "CHAT | {from} | {body}")
                }
            }
            Response::Quit { username } => write!(f, "LEFT | {username}"),
            Response::Success => write!(f, "OK | Success"),
            Response::Join { username } => write!(f, "JOIN | {username}"),
        }
    }
}
