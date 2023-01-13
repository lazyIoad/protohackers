use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader, AsyncBufReadExt, AsyncWrite},
    net::{TcpListener, TcpStream},
};

#[derive(Deserialize)]
struct Request {
    method: String,
    number: f64,
}

#[derive(Serialize)]
struct Response {
    method: String,
    prime: bool,
}

pub async fn start_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut (read_stream, write_stream) = socket.split();
            let buf_reader = BufReader::new(&mut read_stream);
            let mut lines = buf_reader.lines();

            while let Some(line) = lines.next_line().await.unwrap() {
                handle_request(&line, &mut write_stream).await
            }
        });
    }
}

async fn handle_request(body: &str, socket: &mut TcpStream) {
    println!("{}", body);
    let (response, disconnect) = match serde_json::from_str::<Request>(body) {
        Ok(req) => (build_response(req), false),
        Err(_) => (String::new(), true),
    };

    if let Err(e) = socket.write_all(response.as_bytes()).await {
        eprintln!("failed to write to socket: {:?}", e);
        return;
    }

    if disconnect {
        drop(socket);
    }
}

fn build_response(req: Request) -> String {
    let res = Response { method: String::from("isPrime"), prime: is_prime(req.number) };
    serde_json::to_string(&res).expect("Failed to serialize response")
}

fn is_prime(n: i64) -> bool {
    true
}

