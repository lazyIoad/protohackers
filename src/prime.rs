use std::str::FromStr;

use futures::prelude::*;
use num_bigint::BigInt;
use num_traits::{FromPrimitive, One, Zero};
use serde::{Deserialize, Serialize};
use serde_json::Number;
use tokio::net::ToSocketAddrs;
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::{Framed, LinesCodec};

#[derive(Deserialize, Debug)]
struct Request {
    method: String,
    number: Number,
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
        prime: is_number_prime(&req.number),
    };

    serde_json::to_string(&res).expect("Failed to serialize response")
}

fn is_number_prime(n: &Number) -> bool {
    if n.is_f64() {
        return n
            .as_f64()
            .filter(|x| x.fract() < 1e-10)
            .map(|x| BigInt::from_f64(x))
            .flatten()
            .map(|x| is_int_prime(&x))
            .unwrap_or(false);
    }

    return BigInt::from_str(&n.to_string())
        .map(|x| is_int_prime(&x))
        .unwrap_or(false);
}

fn is_int_prime(n: &BigInt) -> bool {
    if *n <= BigInt::one() {
        return false;
    }

    for i in num_iter::range_inclusive(BigInt::from(2), n.sqrt() + BigInt::one()) {
        if Zero::is_zero(&(n % i)) {
            return false;
        }
    }

    true
}
