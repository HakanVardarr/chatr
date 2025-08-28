use std::sync::Arc;
use std::time::Duration;

use crate::client::handle_client;
use crate::command::Command;
use crate::server::run_server;

use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio_rustls::TlsAcceptor;

mod client;
mod command;
mod error;
mod response;
mod server;

const ADDR: &str = "0.0.0.0:3030";
const MAX_USER_SIZE: usize = 50;

pub async fn run() -> anyhow::Result<()> {
    let certs = CertificateDer::pem_file_iter("cert.pem")?.collect::<Result<Vec<_>, _>>()?;
    let key = PrivateKeyDer::from_pem_file("key.pem")?;

    let config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)?;

    let acceptor = TlsAcceptor::from(Arc::new(config));
    let listener = TcpListener::bind(ADDR).await?;

    let (tx, rx) = mpsc::channel::<Command>(MAX_USER_SIZE);
    tokio::spawn(run_server(rx));

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
