use server::run;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    match run().await {
        Ok(()) => {}
        Err(e) => eprintln!("{e}"),
    }

    Ok(())
}
