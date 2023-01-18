use futures::prelude::*;
use serde::{Deserialize, Serialize};
use tokio::net::ToSocketAddrs;
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::{Framed, LinesCodec};

#[derive(Deserialize, Debug)]
struct Request {
    method: String,
    number: f64,
}

#[derive(Serialize)]
struct Response {
    method: String,
    prime: bool,
}

pub async fn start_server(address: impl ToSocketAddrs) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(address).await?;
    loop {
        let (socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            handle_socket(socket).await;
        });
    }
}

async fn handle_socket(socket: TcpStream) {
    let framed_socket = Framed::new(socket, LinesCodec::new());
    let (mut sink, mut stream) = framed_socket.split();

    while let Some(request) = stream.try_next().await.unwrap() {
        let disconnect = handle_request(&request, &mut sink).await;
        if disconnect {
            return;
        }
    }
}

async fn handle_request(
    s: &str,
    sink: &mut stream::SplitSink<Framed<TcpStream, LinesCodec>, String>,
) -> bool {
    let (response, disconnect) = match serde_json::from_str::<Request>(s) {
        Ok(req) => {
            if is_valid_request(&req) {
                (build_response(req), false)
            } else {
                (String::from("invalid request\n"), true)
            }
        }
        Err(_) => (String::from("malformed request\n"), true),
    };

    sink.send(response)
        .await
        .expect("failed to write to socket");

    disconnect
}

fn is_valid_request(req: &Request) -> bool {
    req.method == "isPrime"
}

fn build_response(req: Request) -> String {
    let res = Response {
        method: String::from("isPrime"),
        prime: is_prime(req.number),
    };
    serde_json::to_string(&res).expect("Failed to serialize response")
}

fn is_prime(n: f64) -> bool {
    if n.fract() >= 1e-10 {
        return false;
    }

    // let n = n.trunc() as i64;
    let n_int = n.trunc() as i64;

    if n <= 1. {
        return false;
    }

    for i in 2..(n.sqrt().trunc() as i64 + 1) {
        if n_int % i == 0 {
            return false;
        }
    }

    true
}
