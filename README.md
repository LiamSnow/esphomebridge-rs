# ESPHome Bridge

This is a complete rewrite of aioesphomeapi. It works similarly to aioesphomeapi,
but has a big emphasis on efficiency and async/await. The following features
are not implemented:
 - Bluetooth
 - Voice Assistants

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
