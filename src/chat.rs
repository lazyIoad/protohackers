use tokio::net::ToSocketAddrs;

pub async fn start_server(address: impl ToSocketAddrs) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
