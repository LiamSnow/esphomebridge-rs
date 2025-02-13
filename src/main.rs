use std::{collections::HashMap, error::Error, time::Duration};
use connection::base::Connection;
use device::{Device, DeviceError};
use tokio::task::JoinSet;

pub mod connection;
pub mod device;
pub mod model;
pub mod entity;
pub mod api {
    include!(concat!(env!("OUT_DIR"), "/_.rs"));
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("running");
    let mut devices = HashMap::new();
    devices.insert("crib_b", Device::new_noise("".to_string(), "".to_string()).await?);
    devices.insert("crib_a", Device::new_noise("".to_string(), "".to_string()).await?);

    // devices.insert("speaker_bar", Device::new_noise("".to_string(), "".to_string()).await?);
    // devices.insert("towers", Device::new_noise("".to_string(), "".to_string()).await?);

    devices.insert("living_a", Device::new_noise("".to_string(), "".to_string()).await?);
    devices.insert("living_b", Device::new_noise("".to_string(), "".to_string()).await?);

    devices.insert("kitchen_ceiling", Device::new_noise("".to_string(), "".to_string()).await?);
    devices.insert("kitchen_pantry", Device::new_noise("".to_string(), "".to_string()).await?);
    devices.insert("kitchen_sink", Device::new_noise("".to_string(), "".to_string()).await?);

    let mut set = JoinSet::new();
    for (device_name, device) in devices {
        set.spawn(async move {
            test(device_name.to_string(), device).await
        });
    }
    set.join_all().await;

    Ok(())
}

async fn test<T: Connection>(device_name: String, mut device: Device<T>) -> Result<(), DeviceError> {
    println!("connecting to {device_name}...");
    device.connect().await?;
    let mut req = api::LightCommandRequest {
        key: device.entities.lights.iter().next().unwrap().1.info.key,
        has_state: true,
        state: true,
        has_brightness: true,
        brightness: 1.0,
        has_color_mode: false,
        color_mode: 0,
        has_color_brightness: false,
        color_brightness: 1.,
        has_rgb: false,
        red: 0.,
        green: 1.,
        blue: 0.,
        has_white: false,
        white: 0.,
        has_color_temperature: true,
        color_temperature: 6536.,
        has_cold_white: false,
        cold_white: 1.,
        has_warm_white: false,
        warm_white: 0.,
        has_transition_length: true,
        transition_length: 0,
        has_flash_length: false,
        flash_length: 0,
        has_effect: false,
        effect: "".to_string()
    };

    for i in 0..2 {
        req.brightness = if i % 2 == 0 { 0. } else { 1.0 };
        device.light_command(req.clone()).await?;
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    println!("{device_name} done.");
    Ok(())
}
