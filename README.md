# Poster-rs 📬
## MQTT5 client library

Poster-rs is an asynchronous, runtime agnostic, zero-copy MQTT 5 library,
designed having operation locality in mind.

## Features

- MQTTv5
- Runtime agnostic
- Zero-copy
- Per-subscription async streams
- No unsafe code

### Getting started

Firstly, choose your async runtime. Ready? Lets go!

In the below example we will use Tokio.

```rust
use std::error::Error;
use poster::{prelude::*, ConnectOpts, Context};
use tokio::net::TcpStream;
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let (mut ctx, mut handle) = Context::new();

    let ctx_task = tokio::spawn(async move {
        // Set up a connection using your async framework of choice. We will need a read end, which is
        // AsyncRead, and write end, which is AsyncWrite, so we split the TcpStream
        // into ReadHalf and WriteHalf pair.
        let (rx, tx) = TcpStream::connect("127.0.0.1:1883").await?.into_split();

        // Pass (ReadHalf, WriteHalf) pair into the context and connect with the broker on
        // the protocol level.
        ctx.set_up((rx.compat(), tx.compat_write())).connect(ConnectOpts::default()).await?;

        // Awaiting the Context::run invocation will block the current task.
        if let Err(err) = ctx.run().await {
            eprintln!("[context] Error occured: \"{}\", exiting...", err);
        } else {
            println!("[context] Context exited.");
        }

         Ok::<(), Box<dyn Error + Send + Sync>>(())
    });

    /* ... */

    ctx_task.await?;
    Ok(())
}
```

At this point, our [context](crate::Context) is up and running.

Let's break down the above example.
`poster-rs` is a runtime agnostic library, which means that all the asynchronous operations are abstracted
using traits from the `futures-rs` crate. The result of this approach is that connection with the broker
must be established manually and the library only cares about receving ([AsyncRead](https://docs.rs/futures/latest/futures/io/trait.AsyncRead.html),
[AsyncWrite](https://docs.rs/futures/latest/futures/io/trait.AsyncWrite.html)) pair during the context creation.
This pair is usually obtained using some sort of `split` functions on streams/sockets in the networking libraries.
(See [tokio](https://docs.rs/tokio/latest/tokio/net/struct.TcpStream.html#method.into_split), [smol](https://docs.rs/smol/latest/smol/io/fn.split.html))

new factory method gives us (Context, ContextHandle) tuple. Context is responsible
for handling the traffic between the client and the server. ContextHandle however, is a cloneable handle
to the Context actor and is used to perform all the MQTT operations.

Method run blocks the task (on .await) until one of the following conditions is met:
1. Graceful disconnection is performed (using ContextHandle::disconnect method).
   The result is then ().
2. Error occurs, resulting in MqttError. This may be the result of socket closing,
   receiving DISCONNECT from the server, etc.

### Publishing

Publishing is performed via the ContextHandle::publish method.

```rust
// ...
let opts = PublishOpts::default().topic("topic").data("hello there".as_bytes());
handle.publish(opts).await?;
```

### Subscriptions

Subscriptions are represented as async streams, obtained via the stream.
The general steps of subscribing are:
- await the invocation of ContextHandle::subscribe method
- validate the result (optionally)
- use stream method in order to create a stream for
the subscription.

Note that under the hood, the library uses subscription identifiers to group subscriptions.

```rust
// ...
let opts = SubscribeOpts::default().subscription("topic", SubscriptionOptions::default());
let rsp = handle.subscribe(opts).await?;
let mut subscription = rsp.stream();

while let Some(msg) = subscription.next().await {
    println!("topic: {}; payload: {}", msg.topic_name(), str::from_utf8(msg.payload()).unwrap());
}
```

User may subscribe to multiple topics in one subscription request.

```rust
// ...
let opts = SubscribeOpts::default()
    .subscription("topic1", SubscriptionOptions::default())
    .subscription("topic2", SubscriptionOptions::default());

let mut subscription = handle.subscribe(opts).await?.stream();

while let Some(msg) = subscription.next().await {
    println!("topic: {}; payload: {}", msg.topic_name(), str::from_utf8(msg.payload()).unwrap());
}
```

Each subscription may be customized using the SubscriptionOptions.

```rust
let opts = SubscribeOpts::default().subscription("topic", SubscriptionOptions {
    maximum_qos: QoS::AtLeastOnce,
    no_local: false,
    retain_as_published: true,
    retain_handling: RetainHandling::SendOnSubscribe,
});
```

SubscribeRsp struct represents the result of the subscription request. In order to access
per-topic reason codes, SubscribeRsp::payload method is used:

```rust
// ...
let rsp = handle.subscribe(opts).await?;
let all_ok = rsp.payload().iter().copied().all(|reason| reason == SubackReason::GranteedQoS2);
```

### Unsubscribing

Unsubscribing is performed by the ContextHandle::unsubscribe method.
Note that it does NOT close the subscription stream (it could lead to logic errors).

```rust
// ...
let opts = UnsubscribeOpts::default().topic("topic");
let rsp = handle.unsubscribe(opts).await?;
```

As with subscribing, per topic reason codes can be obtained by the UnsubscribeRsp::payload method.

### Keep alive and ping

If the ConnectOpts::keep_alive interval is set during the connection request,
the user must use the ContextHandle::ping method periodically.

### Disconnection

Disconnection may be initiated either by user or the broker. When initiated by the broker, the Context::run method
returns error::Disconnected error.

Graceful disconnection may be also performed by the user by using ContextHandle::disconnect method.
When disconnection is finished, Context::run method returns ().

```rust
// ...
handle.disconnect(DisconnectOpts::default()).await?;
```

### Error handling

The main library error type is error::MqttError enum found in error module.

### TSL/SSL

TSL/SSL libraries are available out there with AsyncRead, AsyncWrite TSL/SSL streams. These may be
supplied to the Context::set_up method. The library does not handle encription on its own.

## Dependencies

Poster-rs depends on the below crates:

- [futures](https://docs.rs/futures/latest/futures/) - Enables runtime agnostic API
- [bytes](https://docs.rs/bytes/latest/bytes/) - Raw data and buffer management
- [either](https://docs.rs/either/latest/either/) - Utility for handling "unions" of two different types
- [derive_builder](https://docs.rs/derive_builder/latest/derive_builder/) - Implements Builder pattern without code bloat

## License

Copyright 2023 Borys Chyliński

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

## Authors

[Borys Chyliński](https://github.com/Chylynsky)
