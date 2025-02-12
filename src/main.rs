use std::error::Error;
use connection::noise::NoiseConnection;
use device::Device;

pub mod connection;
pub mod device;
pub mod model;
pub mod entity;
pub mod api {
    include!(concat!(env!("OUT_DIR"), "/_.rs"));
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // let mut con = NoiseConnection::new("192.168.1.206:6053", "633FR9s4vpt3FPhCikg4hfYh4YJwCryVeQaifKJu0dM=".to_string())?;
    let conn = NoiseConnection::new("192.168.1.31:6053", "gYytPPML2H1OMNLjsfaD0WCa0pbs/EZvUVpAkAJVmiU=".to_string()).await?;
    let mut device = Device::new(conn).await?;

    loop {
        device.process_incoming().await?;
        for (_,light) in &device.entities.lights {
            if let Some(state) = &light.state {
                println!("{:#?}", state);
                return Ok(())
            }
        }
    }
}
