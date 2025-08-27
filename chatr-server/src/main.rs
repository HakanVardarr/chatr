// CLIENT:
//
// HELLO   | <username>
// MESSAGE | <msg>
// QUIT    |
//
// SERVER:
//
// WELCOME | <username>   | <user_count>
// CHAT    | <username    | <message>
// ERROR   | <error_code> | <error_message>
// LEFT    | <username>

use chatr_server::run;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    run().await
}
