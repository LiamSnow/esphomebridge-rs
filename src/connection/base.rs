use bytes::BytesMut;
use thiserror::Error;

use crate::model::MessageType;

#[derive(Error, Debug)]
pub enum ConnectionError {
    #[error("not connected")]
    NotConnected,
    #[error("unknown message type `{0}`")]
    UnknownMessageType(u16),
    #[error("noise decrypt error `{0}`")]
    NoiseDecryptError(snow::error::Error),
    #[error("tcp io error `{0}`")]
    TcpIOError(std::io::Error),
    #[error("base64 decode slice error `{0}` (noise_psk may be incorrectly sized)")]
    Base64DecodeSliceError(base64::DecodeSliceError),
    #[error("client wants unknown noise protocol `{0}`")]
    ClientWantsUnknownNoiseProtocol(u8),
    #[error("recieved message missing null terminator")]
    MessageMissingNullTerminator,
    #[error("handshake had wrong preamble `{0}`")]
    HandshakeHadWrongPreamble(u8),
    #[error("frame had wrong preamble `{0}`")]
    FrameHadWrongPreamble(u8),
}

impl From<std::io::Error> for ConnectionError {
    fn from(value: std::io::Error) -> Self {
        Self::TcpIOError(value)
    }
}

impl From<snow::error::Error> for ConnectionError {
    fn from(value: snow::error::Error) -> Self {
        Self::NoiseDecryptError(value)
    }
}

impl From<base64::DecodeSliceError> for ConnectionError {
    fn from(value: base64::DecodeSliceError) -> Self {
        Self::Base64DecodeSliceError(value)
    }
}

#[allow(async_fn_in_trait)]
pub trait Connection {
    async fn send_message(&mut self, msg_type: MessageType, msg_bytes: &BytesMut) -> Result<(), ConnectionError>;
    async fn receive_message(&mut self) -> Result<(MessageType, BytesMut), ConnectionError>;
    async fn buffer_empty(&mut self) -> bool;
    async fn connect(&mut self) -> Result<(), ConnectionError>;
    async fn disconnect(&mut self) -> Result<(), ConnectionError>;
}
