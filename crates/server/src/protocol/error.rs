#[derive(Debug, Clone, Copy)]
pub enum ProtocolError {
    InvalidFormat,    // 01
    InvalidCommand,   // 02
    InvalidUsername,  // 03
    AlreadyValidated, // 04
    UserExists,       // 05
    UserDoesntExists, // 06
    NotValidated,     // 07
    MessageYourself,  // 08
}

impl ProtocolError {
    pub fn code(&self) -> u32 {
        match self {
            ProtocolError::InvalidFormat => 1,
            ProtocolError::InvalidCommand => 2,
            ProtocolError::InvalidUsername => 3,
            ProtocolError::AlreadyValidated => 4,
            ProtocolError::UserExists => 5,
            ProtocolError::UserDoesntExists => 6,
            ProtocolError::NotValidated => 7,
            ProtocolError::MessageYourself => 8,
        }
    }

    pub fn message(&self) -> &'static str {
        match self {
            ProtocolError::InvalidFormat => "Please follow protocol.",
            ProtocolError::InvalidCommand => "Invalid command.",
            ProtocolError::InvalidUsername => "Invalid username.",
            ProtocolError::AlreadyValidated => "You are already validated.",
            ProtocolError::UserExists => "User already exists.",
            ProtocolError::UserDoesntExists => "Use doesn't exists.",
            ProtocolError::NotValidated => "Please validate yourself.",
            ProtocolError::MessageYourself => "You cannot messege to yourself.",
        }
    }
}
