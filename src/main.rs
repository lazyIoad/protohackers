mod smoke;

#[tokio::main]
async fn main() {
    tokio::spawn(async { smoke::start(8000).await.unwrap() })
        .await
        .unwrap();
}
