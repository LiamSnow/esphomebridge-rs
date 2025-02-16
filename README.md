# ESPHome Bridge

This is a complete rewrite of aioesphomeapi. It works similarly to aioesphomeapi,
but has a big emphasis on efficiency and async/await. The following features
are not implemented:
 - Bluetooth
 - Voice Assistants

## Usage

```rust
let dev = Device::new_noise("IP", "NOISE_PSK")?;

//if you have multiple devices, you can connect
//to them in parallel using tokio::task::JoinSet
dev.connect().await?;

//turn the light on
let req = api::LightCommandRequest {
    key: device.first_light_key(),
    has_state: true,
    state: true
    ...
};
dev.light_command(req).await?;
```
