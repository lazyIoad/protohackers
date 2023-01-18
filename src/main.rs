use std::env;

use tokio::task::JoinSet;

mod prime;
mod smoke;

#[tokio::main]
async fn main() {
    let host = env::args().skip(1).next().unwrap();
    let mut join_set = JoinSet::new();

    join_set.spawn({
        let host = host.clone();
        async move { smoke::start_server(build_addr(host, 8000)).await.unwrap() }
    });

    join_set.spawn({
        let host = host.clone();
        async move { prime::start_server(build_addr(host, 8001)).await.unwrap() }
    });

    while let Some(_) = join_set.join_next().await {}
}

fn build_addr(host: String, port: usize) -> String {
    format!("{}:{}", host, port)
}
