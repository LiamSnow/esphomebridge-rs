use std::{error::Error, f32::consts::PI, thread, time::{Duration, Instant}};

use connection::noise::NoiseConnection;
use device::Device;

pub mod connection;
pub mod device;
pub mod model;
pub mod entity;
pub mod api {
    include!(concat!(env!("OUT_DIR"), "/_.rs"));
}

fn main() -> Result<(), Box<dyn Error>> {
    // let mut con = NoiseConnection::new("192.168.1.206:6053", "633FR9s4vpt3FPhCikg4hfYh4YJwCryVeQaifKJu0dM=".to_string())?;
    let conn = NoiseConnection::new("192.168.1.31:6053", "gYytPPML2H1OMNLjsfaD0WCa0pbs/EZvUVpAkAJVmiU=".to_string())?;
    let mut device = Device::new(conn)?;

    println!("starting...");
    loop {
        device.process_incoming()?;
        for (_,light) in &device.entities.lights {
            if let Some(state) = &light.state {
                println!("{:#?}", state);
                return Ok(())
            }
        }
    }

    // let mut key: u32 = 0;
    // for entity in entities {
    //     if entity.name.to_lowercase() == "rgbct_bulb" {
    //         key = entity.key;
    //     }
    // }

    // Ok(())
}
