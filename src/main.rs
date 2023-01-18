use std::env;

mod prime;
mod smoke;

#[tokio::main]
async fn main() {
    let host = env::args().skip(1).next().unwrap();

    tokio::spawn({
        let host = host.clone();
        async move { smoke::start_server(build_addr(host, 8000)).await.unwrap() }
    });

    tokio::spawn({
        let host = host.clone();
        async move { prime::start_server(build_addr(host, 8001)).await.unwrap() }
    })
    .await
    .unwrap();
}

fn build_addr(host: String, port: usize) -> String {
    format!("{}:{}", host, port)
}
