use std::{collections::BTreeMap, error::Error, fmt, io};

use bytes::{Buf, BytesMut};
use futures::TryStreamExt;
use std::ops::Bound::Included;
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream, ToSocketAddrs},
};
use tokio_util::codec::{Decoder, FramedRead};

const MESSAGE_LEN: usize = 9;

struct InsertRequest {
    timestamp: i32,
    price: i32,
}

struct QueryRequest {
    mintime: i32,
    maxtime: i32,
}

enum Request {
    Insert(InsertRequest),
    Query(QueryRequest),
}

struct MessageCodec;

impl MessageCodec {
    fn decode_data(&self, src: &mut BytesMut) -> Result<Option<Request>, MessageCodecError> {
        if src.len() < MESSAGE_LEN {
            return Ok(None);
        }

        match src.get_u8() as char {
            'I' => Ok(Some(Request::Insert(InsertRequest {
                timestamp: src.get_i32(),
                price: src.get_i32(),
            }))),
            'Q' => Ok(Some(Request::Query(QueryRequest {
                mintime: src.get_i32(),
                maxtime: src.get_i32(),
            }))),
            _ => Err(MessageCodecError::InvalidMessageType),
        }
    }
}

impl Decoder for MessageCodec {
    type Item = Request;

    type Error = MessageCodecError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        return match self.decode_data(src)? {
            Some(r) => {
                src.reserve(MESSAGE_LEN);
                Ok(Some(r))
            }
            None => Ok(None),
        };
    }
}

#[derive(Debug)]
pub enum MessageCodecError {
    InvalidMessageType,
    Io(io::Error),
}

impl fmt::Display for MessageCodecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessageCodecError::InvalidMessageType => {
                write!(f, "unrecognized message type - must be 'I' or 'Q'")
            }
            MessageCodecError::Io(e) => write!(f, "{}", e),
        }
    }
}

impl From<io::Error> for MessageCodecError {
    fn from(e: io::Error) -> MessageCodecError {
        MessageCodecError::Io(e)
    }
}

impl std::error::Error for MessageCodecError {}

struct Connection {
    socket: TcpStream,
    data: BTreeMap<i32, i32>,
}

impl Connection {
    pub fn new(socket: TcpStream) -> Self {
        Self {
            socket,
            data: BTreeMap::new(),
        }
    }
}

pub async fn start_server(address: impl ToSocketAddrs) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(address).await?;
    loop {
        let (socket, _) = listener.accept().await?;

        let mut c = Connection::new(socket);

        tokio::spawn(async move {
            handle_socket(&mut c).await;
        });
    }
}

async fn handle_socket(c: &mut Connection) {
    let (stream, mut sink) = c.socket.split();
    let mut framed_socket = FramedRead::new(stream, MessageCodec);

    while let Some(request) = framed_socket.try_next().await.unwrap() {
        match &request {
            Request::Insert(insert) => {
                c.data.insert(insert.timestamp, insert.price);
            }
            Request::Query(query) => {
                let mut len = 0;
                let sum = c
                    .data
                    .range((Included(&query.mintime), Included(&query.maxtime)))
                    .map(|(_, v)| v)
                    .fold(0, |acc, x| {
                        len += 1;
                        acc + x
                    });

                if let Err(_) = sink.write_i32(sum / len).await {
                    eprintln!("Failed to write response");
                }
            }
        }
    }
}
