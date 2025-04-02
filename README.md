# ESPHome Bridge

A client API to interact with ESPHome devices (in the same way Home Assistant does).

The following features are **not** implemented:
 - Bluetooth
 - Voice Assistants

[aioesphomeapi](github.com/esphome/aioesphomeapi) was used a reference, but this
is not a one-to-one copy.

## Usage

```rust
let dev = Device::new_noise("IP", "NOISE_PSK")?;
dev.connect().await?;

// print info of all buttons
for e in &dev.entities.button {
    println!("Button: {:#?}", e.1);
}

//turn the light on
let req = api::LightCommandRequest {
    key: dev.entities.light.get("rgbct_bulb").unwrap().key,
    has_state: true,
    state: true
    ..Default::default()
};

dev.light_command(req).await?;
```
