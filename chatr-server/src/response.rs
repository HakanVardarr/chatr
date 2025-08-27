#[derive(Clone, Debug)]
pub enum Response {
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
    },
    Quit {
        username: String,
    },
}
