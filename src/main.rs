use std::env;

use tokio::task::JoinSet;

mod chat;
mod means;
mod prime;
mod smoke;

#[tokio::main]
async fn main() {
    let host = env::args()
        .skip(1)
        .next()
        .unwrap_or(String::from("127.0.0.1"));
    let mut join_set = JoinSet::new();

    join_set.spawn({
        let host = host.clone();
        async move { smoke::start_server(build_addr(host, 8000)).await.unwrap() }
    });

    join_set.spawn({
        let host = host.clone();
        async move { prime::start_server(build_addr(host, 8001)).await.unwrap() }
    });

    join_set.spawn({
        let host = host.clone();
        async move { means::start_server(build_addr(host, 8002)).await.unwrap() }
    });

    join_set.spawn({
        let host = host.clone();
        async move { chat::start_server(build_addr(host, 8003)).await.unwrap() }
    });

    while let Some(_) = join_set.join_next().await {}
}

fn build_addr(host: String, port: usize) -> String {
    format!("{}:{}", host, port)
}
