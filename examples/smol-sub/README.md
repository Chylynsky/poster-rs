# poster-rs using smol

This is a subscription demo app using poster-rs with smol runtime.

Usage:

```
MQTTv5 client library written in Rust.

Usage: smol-sub [OPTIONS] --host <HOST> --topic <TOPIC>

Options:
      --host <HOST>          Broker IP
      --port <PORT>          Broker port [default: 1883]
      --topic <TOPIC>        Topic
      --username <USERNAME>  Username
      --password <PASSWORD>  Password
  -h, --help                 Print help
  -V, --version              Print version
```

Or simply via the cargo run:

```
cargo run --example smol-sub --host 192.168.0.109 --topic "example/#"
```
