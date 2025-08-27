#[derive(Clone, Debug)]
pub enum Response {
    Success,
    Error {
        error_code: u32,
        error_message: String,
    },
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
}
