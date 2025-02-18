use bytes::BytesMut;
use crate::{error::ConnectionError, model::MessageType};

use super::{noise::NoiseConnection, plain::PlainConnection};

#[allow(async_fn_in_trait)]
pub trait Connection {
    async fn send_message(&mut self, msg_type: MessageType, msg_bytes: &BytesMut) -> Result<(), ConnectionError>;
    async fn receive_message(&mut self) -> Result<(MessageType, BytesMut), ConnectionError>;
    async fn buffer_empty(&mut self) -> bool;
    async fn connect(&mut self) -> Result<(), ConnectionError>;
    async fn disconnect(&mut self) -> Result<(), ConnectionError>;
}

#[derive(Hash)]
pub enum AnyConnection {
    Noise(NoiseConnection),
    Plain(PlainConnection)
}

impl From<NoiseConnection> for AnyConnection {
    fn from(value: NoiseConnection) -> Self {
        Self::Noise(value)
    }
}

impl From<PlainConnection> for AnyConnection {
    fn from(value: PlainConnection) -> Self {
        Self::Plain(value)
    }
}

impl Connection for AnyConnection {
    async fn send_message(&mut self, msg_type: MessageType, msg_bytes: &BytesMut) -> Result<(), ConnectionError> {
        match self {
            AnyConnection::Noise(con) => con.send_message(msg_type, msg_bytes).await,
            AnyConnection::Plain(con) => con.send_message(msg_type, msg_bytes).await
        }
    }

    async fn receive_message(&mut self) -> Result<(MessageType, BytesMut), ConnectionError> {
        match self {
            AnyConnection::Noise(con) => con.receive_message().await,
            AnyConnection::Plain(con) => con.receive_message().await
        }
    }

    async fn buffer_empty(&mut self) -> bool {
        match self {
            AnyConnection::Noise(con) => con.buffer_empty().await,
            AnyConnection::Plain(con) => con.buffer_empty().await
        }
    }

    async fn connect(&mut self) -> Result<(), ConnectionError> {
        match self {
            AnyConnection::Noise(con) => con.connect().await,
            AnyConnection::Plain(con) => con.connect().await
        }
    }

    async fn disconnect(&mut self) -> Result<(), ConnectionError> {
        match self {
            AnyConnection::Noise(con) => con.disconnect().await,
            AnyConnection::Plain(con) => con.disconnect().await
        }
    }
}
