use std::{
    collections::HashSet,
    error::Error,
    io::{self, ErrorKind},
    sync::{Arc, Mutex},
};

use futures::{prelude::*, stream::SplitStream};
use futures::{stream::SplitSink, StreamExt};
use itertools::Itertools;
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
    Text { name: String, body: String },
}

#[derive(Debug)]
struct Client {
    name: Option<String>,
    reader: SplitStream<Framed<TcpStream, LinesCodec>>,
    writer: SplitSink<Framed<TcpStream, LinesCodec>, String>,
    server_sender: Sender<Message>,
    server_receiver: Receiver<Message>,
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

async fn handle_client(
    socket: TcpStream,
    chat_tx: Sender<Message>,
    server_data: ServerData,
) -> Result<(), Box<dyn Error>> {
    let receiver = chat_tx.subscribe();
    let framed_socket = Framed::new(socket, LinesCodec::new());
    let (client_writer, client_reader) = framed_socket.split();

    let mut client_state = Client {
        name: None,
        reader: client_reader,
        writer: client_writer,
        server_sender: chat_tx,
        server_receiver: receiver,
    };

    client_state.writer.send(WELCOME_MSG.to_owned()).await?;

    save_client_name(&server_data, &mut client_state).await?;

    send_presence_msg(&server_data, &mut client_state).await;

    client_state.server_receiver = client_state.server_receiver.resubscribe();

    loop {
        tokio::select! {
            // Receive an event from another client
            Ok(msg) = client_state.server_receiver.recv() => {
                handle_client_message(&msg, &mut client_state, &server_data).await;
            }

            // Receive a message from this client
            Ok(val) = client_state.reader.try_next() => {
                if let Some(msg) = val {
                    if let Some(name) = &client_state.name {
                        client_state.server_sender.send(Message::Text {name: name.clone(), body: msg}).unwrap();
                    }
                } else {
                    if let Some(name) = &client_state.name {
                        client_state.server_sender.send(Message::Leave(name.clone())).unwrap();
                        let mut server_data = server_data.lock().unwrap();
                        server_data.remove(name);
                    }

                    return Ok(());
                }
            }
        }
    }
}

async fn save_client_name(
    server_data: &ServerData,
    client_state: &mut Client,
) -> Result<(), Box<dyn Error>> {
    // stream.try_next().await?.map()
    if let Some(name) = client_state.reader.try_next().await? {
        {
            if name.is_empty() || !name.chars().all(char::is_alphanumeric) {
                return Err(Box::new(io::Error::new(
                    ErrorKind::InvalidInput,
                    "name must be non-empty and alphanumeric",
                )));
            }

            let mut server_data = match server_data.lock() {
                Ok(d) => d,
                Err(e) => todo!(),
            };

            if server_data.contains(&name) {
                // Disallow duplicate names
                return Err(Box::new(io::Error::new(
                    ErrorKind::InvalidInput,
                    "duplicate name",
                )));
            }

            server_data.insert(name.clone());
        }

        client_state.name = Some(name.clone());
        client_state.server_sender.send(Message::Join(name))?;

        return Ok(());
    } else {
        Err(Box::new(io::Error::new(
            ErrorKind::ConnectionReset,
            "client disconnected before name was given",
        )))
    }
}

async fn send_presence_msg(server_data: &ServerData, state: &mut Client) {
    let a: String;
    {
        let server_data = server_data.lock().unwrap();
        let mut names = Itertools::join(
            &mut server_data.iter().filter(|name| {
                state
                    .name
                    .as_ref()
                    .map_or(false, |client_name| *name != client_name)
            }),
            ", ",
        );

        if names.is_empty() {
            names = String::from("just you!");
        }

        a = format!("* The room contains: {}", names);
    }
    state.writer.send(a).await.unwrap();
}

async fn handle_client_message(msg: &Message, state: &mut Client, server_data: &ServerData) {
    match msg {
        Message::Join(name) => {
            if let Some(client_name) = &state.name {
                let a = format!("* {} has entered the room", name);
                state.writer.send(a).await.unwrap();
            }
        }
        Message::Leave(name) => {
            if let Some(client_name) = &state.name {
                if client_name != name {
                    let a = format!("* {} has left the room", name);
                    state.writer.send(a).await.unwrap();
                }
            }
        }
        Message::Text { name, body } => {
            if let Some(client_name) = &state.name {
                if client_name != name {
                    let a = format!("[{}] {}", name, body);
                    state.writer.send(a).await.unwrap();
                }
            }
        }
    }
}
