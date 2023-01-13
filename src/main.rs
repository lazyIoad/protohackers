mod smoke;
mod prime;

#[tokio::main]
async fn main() {
    // tokio::spawn(async { smoke::start_server(8000).await.unwrap() });

    tokio::spawn(async { prime::start_server(8001).await.unwrap() })
        .await
        .unwrap();
}
