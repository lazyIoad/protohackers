use std::{error::Error, sync::{Arc, Mutex}, collections::HashSet};

use futures::StreamExt;
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream, ToSocketAddrs},
    sync::broadcast::{self, Receiver, Sender},
};
use tokio_util::codec::{Framed, FramedRead, LinesCodec};

static WELCOME_MSG: &str = "Welcome to budgetchat! What shall I call you?";

type ServerData = Arc<Mutex<HashSet<String>>>;

enum Message {}

struct Client {
    name: String,
    send: Sender<Message>,
    recv: Receiver<Message>,
}

pub async fn start_server(address: impl ToSocketAddrs) -> Result<(), Box<dyn Error>> {
    let server_state: ServerData = Arc::new(Mutex::new(HashSet::new()));

    let listener = TcpListener::bind(address).await?;
    loop {
        let (mut socket, _) = listener.accept().await?;

        // let mut c = Connection::new(socket);
        //
        let (tx, rx1) = broadcast::channel(16);

        tokio::spawn(async move {
            handle_socket(&mut socket).await;
        });
    }
}

async fn handle_socket(socket: &mut TcpStream) {
    let framed_socket = Framed::new(socket, LinesCodec::new());
    let (mut sink, mut stream) = framed_socket.split();
    // socket.write_all(WELCOME_MSG.as_bytes()).await;
    // let (stream, mut sink) = c.socket.split();
    // sink.
    // let mut framed_socket = FramedRead::new(stream, MessageCodec);

    // while let Some(request) = framed_socket.try_next().await.unwrap() {
    //     match &request {
    //         Request::Insert(insert) => {
    //             c.data.insert(insert.timestamp, insert.price);
    //         }
    //         Request::Query(query) => {
    //             let mean = get_mean(&c.data, query);
    //
    //             if let Err(_) = sink.write_i32(mean).await {
    //                 eprintln!("Failed to write response");
    //             }
    //         }
    //     }
    // }
}
