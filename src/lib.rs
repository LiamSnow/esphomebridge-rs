pub mod connection;
pub mod device;
pub mod model;
pub mod entity;
pub mod error;
pub mod api {
    include!(concat!(env!("OUT_DIR"), "/_.rs"));
}


#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::time::timeout;

    use crate::device::ESPHomeDevice;

    #[tokio::test]
    async fn test() {
        let mut dev = ESPHomeDevice::new_noise("192.168.1.18:6053".to_string(), "GwsvILrvcN/BHAG9m7Hgzcqzc4Dx9neT/1RfEDmsecw=".to_string());
        dev.connect().await.unwrap();
        println!("connected");

        let mut rx = dev.subscribe_states(5).await.unwrap();

        loop {
            match timeout(Duration::from_millis(100), rx.recv()).await {
                Ok(Some(update)) => println!("{:#?}", update),
                Ok(None) => {
                    println!("Channel closed");
                    break;
                }
                Err(_) => {
                    dev.process_incoming().await.unwrap();
                    continue;
                }
            }
        }

        // let a = dev.entities.light.get("rgbct_bulb").unwrap();
        //
        // println!("{:#?}", a);


        // let key = dev.entities.light.get("rgbct_bulb").unwrap().key;

        // dev.subscribe_states(
        //     move |states: EntityStates| {
        //         let state = states.light.get(&key);
        //         println!("state:{:#?}", state);
        //     }
        // ).await.unwrap();
        //
        // loop {
        //     dev.process_incoming().await.unwrap();
        //     let _ = tokio::time::sleep(Duration::from_millis(50)).await;
        // }
    }
}
