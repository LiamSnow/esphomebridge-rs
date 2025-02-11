use base64::prelude::*;
use bytes::{Buf, BytesMut};
use memchr::memchr;
use snow::{HandshakeState, TransportState};
use std::{
    error::Error,
    io::{Read, Write},
    net::TcpStream,
    time::Duration,
};
use crate::model::MessageType;
use super::base::Connection;

pub const NOISE_HELLO: &[u8; 3] = b"\x01\x00\x00";
pub const READ_TIMEOUT: Option<Duration> = Some(Duration::from_secs(60));

pub struct NoiseConnection {
    stream: TcpStream,
    noise: TransportState,
    pub server_name: String,
}

impl NoiseConnection {
    pub fn new(ip: &str, noise_psk: String) -> Result<Self, Box<dyn Error>> {
        let mut noise_handshake = Self::setup_noise(noise_psk)?;

        //TODO TCPListener?
        let mut stream = TcpStream::connect(ip)?;
        stream.set_read_timeout(READ_TIMEOUT)?;

        Self::send_hello(&mut stream, &mut noise_handshake)?;
        let server_name = Self::receive_hello(&mut stream)?;
        let noise = Self::receive_handshake(&mut stream, noise_handshake)?;

        Ok(Self {
            stream,
            noise,
            server_name,
        })
    }

    fn setup_noise(noise_psk: String) -> Result<HandshakeState, Box<dyn Error>> {
        let key_vec = BASE64_STANDARD.decode(&noise_psk)?;
        let key: [u8; 32] = key_vec.try_into().map_err(|_| "Invalid key length")?;

        let params = "Noise_NNpsk0_25519_ChaChaPoly_SHA256".parse()?;
        let handshake = snow::Builder::new(params)
            .psk(0, &key) //TODO check location=0
            .prologue(b"NoiseAPIInit\x00\x00")
            .build_initiator()?;

        Ok(handshake)
    }

    /// Send ClientHello to the server
    fn send_hello(
        stream: &mut TcpStream,
        noise_handshake: &mut HandshakeState,
    ) -> Result<(), Box<dyn Error>> {
        let mut frame = BytesMut::with_capacity(65535);
        frame.resize(65535, 0);
        let mut frame_len = noise_handshake.write_message(&[], &mut frame)?;
        frame.truncate(frame_len);

        frame_len += 1;
        let header = [
            0x01,
            ((frame_len >> 8) & 0xFF) as u8,
            (frame_len & 0xFF) as u8,
        ];

        let mut message = BytesMut::with_capacity(3 + 3 + 1 + frame_len);
        message.extend_from_slice(NOISE_HELLO);
        message.extend_from_slice(&header);
        message.extend_from_slice(&[0x00]);
        message.extend_from_slice(&frame);
        stream.write(&message)?;

        Ok(())
    }

    fn receive_hello(stream: &mut TcpStream) -> Result<String, Box<dyn Error>> {
        let frame = Self::read_frame(stream)?;

        let chosen_proto = frame[0];
        if chosen_proto != 0x01 {
            return Err("Wrong chosen protocol in noise hello".into());
        }

        let pos = memchr(0, &frame[1..]).ok_or("No null terminator")?;
        let server_name = String::from_utf8_lossy(&frame[1..pos + 1]).into_owned();
        Ok(server_name)
    }

    fn receive_handshake(
        stream: &mut TcpStream,
        mut noise_handshake: HandshakeState,
    ) -> Result<TransportState, Box<dyn Error>> {
        let frame = Self::read_frame(stream)?;

        let preamble = frame[0];
        if preamble != 0 {
            return Err("Wrong handshake preamble".into());
        }

        let mut payload = BytesMut::with_capacity(2);
        let payload_len = noise_handshake.read_message(&frame[1..], &mut payload)?;
        payload.truncate(payload_len);

        Ok(noise_handshake.into_transport_mode()?)
    }

    fn read_frame(stream: &mut TcpStream) -> Result<BytesMut, Box<dyn Error>> {
        let mut header = [0u8; 3];
        stream.read_exact(&mut header)?;

        let preamble = header[0];
        if preamble != 0x01 {
            return Err("Wrong preamble when reading noise frame".into());
        }

        let frame_size = u16::from_be_bytes([header[1], header[2]]) as usize;
        let mut frame = BytesMut::with_capacity(frame_size);
        frame.resize(frame_size, 0);
        stream.read_exact(&mut frame)?;

        Ok(frame)
    }
}

impl Connection for NoiseConnection {
    fn send_message(
        &mut self,
        req: impl prost::Message,
        msg_type: MessageType,
    ) -> Result<(), Box<dyn Error>> {
        //convert protoc message to bytes
        let data_len = req.encoded_len();
        let mut data = BytesMut::with_capacity(data_len);
        req.encode(&mut data)?;

        //make frame
        let msg_type = msg_type as usize;
        let frame_header = [
            ((msg_type >> 8) & 0xFF) as u8,
            (msg_type & 0xFF) as u8,
            ((data_len >> 8) & 0xFF) as u8,
            (data_len & 0xFF) as u8,
        ];
        let mut frame = BytesMut::with_capacity(frame_header.len() + data.len());
        frame.extend_from_slice(&frame_header);
        frame.extend_from_slice(&data);

        //encrypt frame
        let mut eframe = BytesMut::with_capacity(65535);
        eframe.resize(65535, 0);
        let eframe_len = self.noise.write_message(&frame, &mut eframe)?;
        eframe.truncate(eframe_len);

        //make packet
        let packet_header = [
            0x01,
            ((eframe_len >> 8) & 0xFF) as u8,
            (eframe_len & 0xFF) as u8,
        ];
        let mut packet = BytesMut::with_capacity(packet_header.len() + eframe.len());
        packet.extend_from_slice(&packet_header);
        packet.extend_from_slice(&eframe);

        //send packet
        self.stream.write_all(&packet)?;
        self.stream.flush()?;

        Ok(())
    }

    fn receive_message_raw(&mut self) -> Result<(MessageType, BytesMut), Box<dyn Error>> {
        let frame = Self::read_frame(&mut self.stream)?;
        let mut msg = BytesMut::with_capacity(65535);
        msg.resize(65535, 0);
        let msg_size = self.noise.read_message(&frame, &mut msg)?;
        msg.truncate(msg_size);
        let msg_type = MessageType::from_repr(u16::from_be_bytes([msg[0], msg[1]]))
            .ok_or("unknown message type")?;
        msg.advance(4);
        Ok((msg_type, msg))
    }

    fn receive_message<T: prost::Message + Default>(
        &mut self,
        expected_msg_type: MessageType,
    ) -> Result<T, Box<dyn Error>> {
        let (msg_type, mut msg) = self.receive_message_raw()?;
        if msg_type != expected_msg_type {
            return Err(format!("Wrong message type: got {:#?}", msg_type).into());
        }
        Ok(T::decode(&mut msg)?)
    }

    fn transaction<T: prost::Message + Default>(
        &mut self,
        req: impl prost::Message,
        req_type: MessageType,
        res_type: MessageType,
    ) -> Result<T, Box<dyn Error>> {
        self.send_message(req, req_type)?;
        Ok(self.receive_message(res_type)?)
    }

    fn buffer_empty(&mut self) -> Result<bool, Box<dyn Error>> {
        self.stream.set_read_timeout(Some(Duration::from_millis(10)))?; //FIXME
        let mut buf = [0; 1];
        let res = self.stream.peek(&mut buf).is_err();
        self.stream.set_read_timeout(READ_TIMEOUT)?;
        Ok(res)
    }

    fn disconnect(&mut self) {
        //TODO close TCP
    }
}
