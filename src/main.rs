use anyhow::Result;
use simple_redis::{backend::Backend, network::stream_handler};
use tokio::net::TcpListener;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let addr = "0.0.0.0:6379";
    info!("Simple-Redis-Server listening on {}", addr);
    let listener = TcpListener::bind(addr).await?;

    let backend = Backend::new();

    loop {
        let (stream, raddr) = listener.accept().await?;
        info!("New connection from {}", raddr);

        let backend_cloned = backend.clone();

        tokio::spawn(async move {
            match stream_handler(stream, backend_cloned).await {
                Ok(_) => info!("Connection from {} exited", raddr),
                Err(e) => warn!("Connection closed with error: {}", e),
            }
        });
    }
}
