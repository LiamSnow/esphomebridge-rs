use bytes::BytesMut;
use crate::{error::ConnectionError, model::MessageType};

use super::{noise::NoiseConnection, plain::PlainConnection};

#[allow(async_fn_in_trait)]
pub trait Connection {
    async fn send_message(&mut self, msg_type: MessageType, msg_bytes: &BytesMut) -> Result<(), ConnectionError>;
    async fn receive_message(&mut self, first_byte: Option<u8>) -> Result<(MessageType, BytesMut), ConnectionError>;
    fn try_read_byte(&mut self) -> Result<Option<u8>, ConnectionError>;
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

    async fn receive_message(&mut self, first_byte: Option<u8>) -> Result<(MessageType, BytesMut), ConnectionError> {
        match self {
            AnyConnection::Noise(con) => con.receive_message(first_byte).await,
            AnyConnection::Plain(con) => con.receive_message(first_byte).await
        }
    }

    fn try_read_byte(&mut self) -> Result<Option<u8>, ConnectionError> {
        match self {
            AnyConnection::Noise(con) => con.try_read_byte(),
            AnyConnection::Plain(con) => con.try_read_byte()
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
