use std::sync::Arc;
use std::time::Duration;

use crate::client::handle_client;
use crate::server::Server;

use clap::Parser;
use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio_rustls::TlsAcceptor;

mod client;
mod protocol;
mod server;

const ADDR: &str = "0.0.0.0:3030";
const MAX_USER_SIZE: usize = 50;

#[derive(Parser, Debug)]
#[command(name = "server", version, about = "Tokio + TLS Chat Server")]
struct Cli {
    /// Protocol: tcp or tls
    #[arg(long, default_value = "tcp", value_parser = ["tcp", "tls"])]
    protocol: String,

    /// TLS certificate file (PEM)
    #[arg(long, requires = "key", required_if_eq("protocol", "tls"))]
    cert: Option<String>,

    /// TLS private key file (PEM)
    #[arg(long, requires = "cert", required_if_eq("protocol", "tls"))]
    key: Option<String>,
}

pub async fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let listener = TcpListener::bind(ADDR).await?;
    let (tx, rx) = mpsc::channel(MAX_USER_SIZE);

    let mut server = Server::new(rx);
    tokio::spawn(async move { server.run().await });

    match cli.protocol.as_str() {
        "tcp" => {
            println!("Starting TCP server on {ADDR}");

            loop {
                let (stream, _) = listener.accept().await?;
                let socket_ref = socket2::SockRef::from(&stream);
                let mut ka = socket2::TcpKeepalive::new();

                ka = ka.with_time(Duration::from_secs(20));
                ka = ka.with_interval(Duration::from_secs(20));
                socket_ref.set_tcp_keepalive(&ka)?;

                let sender = tx.clone();
                tokio::spawn(async move {
                    let _ = handle_client(stream, sender).await;
                });

                std::thread::yield_now();
            }
        }
        "tls" => {
            let cert_file = cli.cert.expect("--cert is required with --protocol tls");
            let key_file = cli.key.expect("--key is required with --protocol tls");

            let certs = CertificateDer::pem_file_iter(cert_file)?.collect::<Result<Vec<_>, _>>()?;
            let key = PrivateKeyDer::from_pem_file(key_file)?;

            let config = rustls::ServerConfig::builder()
                .with_no_client_auth()
                .with_single_cert(certs, key)?;
            let acceptor = TlsAcceptor::from(Arc::new(config));

            println!("Starting TLS server on {ADDR}");

            loop {
                let (stream, _) = listener.accept().await?;
                let socket_ref = socket2::SockRef::from(&stream);
                let mut ka = socket2::TcpKeepalive::new();

                ka = ka.with_time(Duration::from_secs(20));
                ka = ka.with_interval(Duration::from_secs(20));
                socket_ref.set_tcp_keepalive(&ka)?;

                let acceptor = acceptor.clone();
                let sender = tx.clone();

                tokio::spawn(async move {
                    if let Ok(tls_stream) = acceptor.accept(stream).await {
                        let _ = handle_client(tls_stream, sender).await;
                    }
                });

                std::thread::yield_now();
            }
        }
        _ => {}
    }

    Ok(())
}
