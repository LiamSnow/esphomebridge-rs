# ESPHome Bridge

A client API to interact with ESPHome devices (in the same way Home Assistant does).

The following features are **not** implemented:
 - Bluetooth
 - Voice Assistants

[aioesphomeapi](github.com/esphome/aioesphomeapi) was used a reference, but this
is not a one-to-one copy.

## Usage

Connect:
```rust
let dev = Device::new_noise("IP", "NOISE_PSK")?;
dev.connect().await?;
```

Print all buttons:
```rust
for e in &dev.entities.button {
    println!("Button: {:#?}", e.1);
}
```

Turn on all lights:
```rust
for light in dev.entities.light {
    let req = api::LightCommandRequest {
        key: light.key,
        has_state: true,
        state: true
        ..Default::default()
    };

    dev.light_command(req).await?;
}
```

Turn on a light, given the entity name
```rust
let req = api::LightCommandRequest {
    key: dev.get_light_key_from_name("rgbct_bulb")?,
    has_state: true,
    state: true
    ..Default::default()
};

dev.light_command(req).await?;
```

See [liamsnow.com](https://liamsnow.com/projects/esphomebridge-rs) for more.
