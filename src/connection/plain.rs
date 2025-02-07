// use std::{error::Error, io::Write, net::TcpStream, time::Duration};
// use bytes::BytesMut;
// use crate::api;
//
// use super::base::Connection;

// pub struct PlainConnection {
//     pub stream: TcpStream,
// }
//
// impl PlainConnection {
//     pub fn new(ip: &str) -> Result<Self, Box<dyn Error>> {
//         let mut me = Self { stream: TcpStream::connect(ip)? };
//         me.stream.set_read_timeout(Some(Duration::from_secs(60)));
//         me.connect();
//         Ok(me)
//     }
//
//     fn connect(&mut self) -> Result<(), Box<dyn Error>> {
//         let req = api::HelloRequest {
//             client_info: "iron-esphome".to_string(),
//             api_version_major: 1,
//             api_version_minor: 10,
//         };
//
//         println!("sending message...");
//         self.send_message(req)?;
//         println!("done");
//
//         let res: api::HelloResponse = self.receive_message()?;
//         println!("got {:#?}", res);
//
//         Ok(())
//     }
// }

// impl Connection for PlainConnection {
//     fn send_message(&mut self, req: impl prost::Message) -> Result<(), Box<dyn Error>> {
//         let mut buf = BytesMut::with_capacity(req.encoded_len());
//         req.encode(&mut buf)?;
//
//         self.stream.write_all(&buf)?;
//         self.stream.flush()?;
//
//         Ok(())
//     }
//
//     fn receive_message<T: prost::Message + Default>(&mut self) -> Result<T, Box<dyn Error>> {
//         self.stream.set_read_timeout(Some(Duration::from_secs(30)))?;
//         // let len = self.stream.read_varint32()?;
//         // let tp = self.stream.read_varint32()?;
//         //
//         // println!("len:{len}, tp:{tp}");
//         //
//         // // Read the message body
//         // let mut buffer = vec![0u8; len as usize];
//         // self.stream.read_exact(&mut buffer)?;
//
//         // Decode the message
//         // let message = T::decode(&buffer[..])?;
//         let message = T::decode(&self.stream)?;
//         Ok(message)
//     }
//
//     fn close(&mut self) {
//         //TODO close TCP
//     }
// }
