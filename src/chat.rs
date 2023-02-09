use std::{
    collections::HashSet,
    error::Error,
    sync::{Arc, Mutex},
};

use futures::prelude::*;
use futures::StreamExt;
use tokio::{
    net::{TcpListener, TcpStream, ToSocketAddrs},
    sync::broadcast::{self, Receiver, Sender},
};
use tokio_util::codec::{Framed, LinesCodec};

static WELCOME_MSG: &str = "Welcome to budgetchat! What shall I call you?";

type ServerData = Arc<Mutex<HashSet<String>>>;

#[derive(Clone, Debug)]
enum Message {
    Join(String),
    Leave(String),
    Message { name: String, body: String },
}

#[derive(Debug)]
struct Client {
    name: Option<String>,
    sender: Sender<Message>,
    receiver: Receiver<Message>,
}

pub async fn start_server(address: impl ToSocketAddrs) -> Result<(), Box<dyn Error>> {
    let server_state: ServerData = Arc::new(Mutex::new(HashSet::new()));
    let listener = TcpListener::bind(address).await?;
    let (tx, _) = broadcast::channel(32);

    loop {
        let (socket, _) = listener.accept().await?;

        let server_state = server_state.clone();
        let tx = tx.clone();

        tokio::spawn(async move {
            handle_client(socket, tx, server_state).await;
        });
    }
}

async fn handle_client(socket: TcpStream, chat_tx: Sender<Message>, server_data: ServerData) {
    let receiver = chat_tx.subscribe();

    let mut client_state = Client {
        name: None,
        sender: chat_tx,
        receiver,
    };

    let framed_socket = Framed::new(socket, LinesCodec::new());
    let (mut sink, mut stream) = framed_socket.split();
    sink.send(WELCOME_MSG.to_owned()).await.unwrap();

    if let Some(name) = stream.try_next().await.unwrap() {
        {
            let mut server_data = server_data.lock().unwrap();
            if server_data.contains(&name) {
                // Disallow duplicate names
                return;
            }

            server_data.insert(name.clone());
        }

        client_state.name = Some(name.clone());
        client_state.sender.send(Message::Join(name)).unwrap();
    }

    loop {
        tokio::select! {
            Ok(val) = client_state.receiver.recv() => {
               match val {
                    Message::Join(name) => {
                        if let Some(client_name) = &client_state.name {
                            if client_name != &name {
                                let a = format!("* {} has entered the room", name);
                                sink.send(a).await.unwrap();
                            } else {
                                // list
                            }
                        }
                    },
                    Message::Leave(name) => {
                        if let Some(client_name) = &client_state.name {
                            if client_name != &name {
                                let a = format!("* {} has left the room", name);
                                sink.send(a).await.unwrap();
                            }
                        }
                    }
                    Message::Message { name, body } => {
                        if let Some(client_name) = &client_state.name {
                            if client_name != &name {
                                let a = format!("[{}] {}", name, body);
                                sink.send(a).await.unwrap();
                            }
                        }
                    },
                }
            }
            Ok(val) = stream.try_next() => {
                if let Some(msg) = val {
                    if let Some(name) = &client_state.name {
                        client_state.sender.send(Message::Message {name: name.clone(), body: msg}).unwrap();
                    }
                } else {
                    if let Some(name) = &client_state.name {
                        client_state.sender.send(Message::Leave(name.clone())).unwrap();
                    }

                    return;
                }
            }
        }
    }
}
