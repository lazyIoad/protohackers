use std::error::Error;
use std::str::FromStr;

use futures::prelude::*;
use num_bigint::{BigInt, ParseBigIntError};
use num_traits::{One, Zero};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::net::ToSocketAddrs;
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::{Framed, LinesCodec};

#[derive(Deserialize)]
struct Request {
    method: String,
    number: String, // handle parsing manually due to arbitrary precision
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
    let data = r#"
        {
            "name": "John Doe",
            "age": 43,
            "phones": [
                "+44 1234567",
                "+44 2345678"
            ]
        }"#;

    // Parse the string of data into serde_json::Value.
    let v: Value = serde_json::from_str(data)?;
    let a = v["name"];
    let (response, disconnect) = match serde_json::from_str::<Request>(s) {
        Ok(req) => {
            if is_valid_request(&req) {
                match build_response(&req) {
                    Ok(res) => (res, false),
                    Err(_) => (String::from("invalid request\n"), true),
                }
            } else {
                (String::from("invalid request\n"), true)
            }
        }
        Err(e) => {
            eprintln!("{}", e);
            (String::from("malformed request\n"), true)
        }
    };

    sink.send(response)
        .await
        .expect("failed to write to socket");

    disconnect
}

fn is_valid_request(req: &Request) -> bool {
    req.method == "isPrime"
}

fn build_response(req: &Request) -> Result<String, Box<dyn Error>> {
    let prime = is_prime(&req.number)?;
    let res = Response {
        method: String::from("isPrime"),
        prime,
    };

    Ok(serde_json::to_string(&res).expect("Failed to serialize response"))
}

fn is_prime(num: &str) -> Result<bool, ParseBigIntError> {
    // Handle negative numbers (in scientific notation) and decimals
    if num.contains('-') || num.contains('.') {
        return Ok(false);
    }

    let num = BigInt::from_str(num)?;

    if num <= BigInt::one() {
        return Ok(false);
    }

    for i in num_iter::range_inclusive(BigInt::from(2), num.sqrt() + BigInt::one()) {
        if (&num % i) == BigInt::zero() {
            return Ok(false);
        }
    }

    Ok(true)
}
