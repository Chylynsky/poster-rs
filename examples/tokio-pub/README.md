# poster-rs using tokio

This is a publish demo app using poster-rs with tokio runtime.

Usage:

```
MQTTv5 client library written in Rust.

Usage: tokio-pub [OPTIONS] --host <HOST> --topic <TOPIC> --message <MESSAGE>

Options:
      --host <HOST>          Broker IP
      --port <PORT>          Broker port [default: 1883]
      --topic <TOPIC>        Topic
      --username <USERNAME>  Username
      --password <PASSWORD>  Password
      --message <MESSAGE>    Message
      --qos <QOS>            Quality of Service [default: 0]
  -h, --help                 Print help
  -V, --version              Print version
```

Or simply via the cargo run:

```
cargo run --example tokio-pub -- --host 192.168.0.109 --topic example --message 'hello there :)'
```
