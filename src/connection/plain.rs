use bytes::{BytesMut, BufMut};
use tokio::io::AsyncReadExt;
use tokio::{net::TcpStream, io::AsyncWriteExt};
use std::hash::{Hash, Hasher};
use crate::{error::ConnectionError, model::MessageType};
use super::base::Connection;
use super::util::{varu32_to_bytes, Varu32};

///NOTE UNTESTED!!!!!!!!
pub struct PlainConnection {
    pub(crate) ip: String,
    pub(crate) stream: Option<TcpStream>,
}

impl Hash for PlainConnection {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ip.hash(state);
    }
}

impl Connection for PlainConnection {
    async fn send_message(&mut self, msg_type: MessageType, msg_bytes: &BytesMut) -> Result<(), ConnectionError> {
        let stream = self.stream.as_mut().ok_or(ConnectionError::NotConnected)?;

        let msg_type_var = varu32_to_bytes(msg_type as u32);
        let msg_len = msg_bytes.len();
        let msg_len_var = varu32_to_bytes(msg_len as u32);

        let mut packet = BytesMut::with_capacity(msg_len + 1 + msg_type_var.len() + msg_len_var.len());
        packet.put_u8(0);
        packet.extend_from_slice(&msg_type_var);
        packet.extend_from_slice(&msg_len_var);
        packet.extend_from_slice(&msg_bytes);

        stream.write_all(&packet).await?;
        stream.flush().await?;

        Ok(())
    }

    async fn receive_message(&mut self, first_byte: Option<u8>) -> Result<(MessageType, BytesMut), ConnectionError> {
        let stream = self.stream.as_mut().ok_or(ConnectionError::NotConnected)?;
        let preamble = stream.read_varu32(first_byte).await?;
        if preamble != 0x00 {
            return Err(ConnectionError::FrameHadWrongPreamble(preamble as u8))
        }

        let msg_len = stream.read_varu32(None).await? as usize;
        let msg_type_num = stream.read_varu32(None).await? as u16;
        let msg_type = MessageType::from_repr(msg_type_num)
            .ok_or(ConnectionError::UnknownMessageType(msg_type_num))?;
        let mut msg = BytesMut::with_capacity(msg_len);
        stream.read_buf(&mut msg).await?;
        Ok((msg_type, msg))
    }

    fn try_read_byte(&mut self) -> Result<Option<u8>, ConnectionError> {
        let stream = self.stream.as_mut().ok_or(ConnectionError::NotConnected)?;
        let mut buf = [0u8; 1];
        if stream.try_read(&mut buf).is_ok() {
            Ok(Some(buf[0]))
        }
        else {
            Ok(None)
        }
    }

    async fn connect(&mut self) -> Result<(), ConnectionError> {
        if self.stream.is_some() {
            return Ok(()) //TODO: is this wanted behavior... should this error? should it reconnect?
        }
        let stream = TcpStream::connect(&self.ip).await?;
        self.stream = Some(stream);
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), ConnectionError> {
        if let Some(stream) = &mut self.stream {
            stream.shutdown().await?;
        }
        self.stream = None;
        Ok(())
    }
}

impl PlainConnection {
    pub fn new(ip: String) -> Self {
        Self { ip, stream: None }
    }
}


