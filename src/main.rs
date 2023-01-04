mod smoke;

#[tokio::main]
async fn main() {
    tokio::spawn(async { smoke::echo::start(8000).await.unwrap() })
        .await
        .unwrap();
}
