use core::MessageType;
use std::error::Error;

use connection::{base::Connection, noise::NoiseConnection};

pub mod connection;
pub mod core;
pub mod api {
    include!(concat!(env!("OUT_DIR"), "/_.rs"));
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut con = NoiseConnection::new("192.168.1.206:6053", "REDACTED".to_string())?;

    let req = api::HelloRequest {
        client_info: "iron-esphome".to_string(),
        api_version_major: 1,
        api_version_minor: 9,
    };
    con.send_message(req, MessageType::HelloRequest)?;

    let req = api::ConnectRequest {
        password: "".to_string()
    };
    con.send_message(req, MessageType::ConnectRequest)?;

    let _: api::HelloResponse = con.receive_message(MessageType::HelloResponse)?;

    if let Ok(res) = con.receive_message::<api::ConnectResponse>(MessageType::ConnectResponse) {
        if res.invalid_password {
            return Err("Invalid password".into());
        }
    }

    Ok(())
}
