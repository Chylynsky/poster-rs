#![forbid(unsafe_code, unreachable_pub, unused_must_use)]
#![warn(missing_docs)]
#![allow(dead_code)]

//! Poster-rs is an asynchronous, runtime agnostic, zero-copy MQTT 5 library,
//! designed having operation locality in mind.
//!
//! ## Set up
//! Firstly, choose your async runtime. Ready? Lets go!
//!
//! In the below example we will use Tokio.  
//!
//! ```no_run
//! use std::error::Error;
//! use poster::{prelude::*, ConnectOpts, Context};
//! use tokio::net::TcpStream;
//! use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};
//!
//! #[tokio::main(flavor = "current_thread")]
//! async fn main() -> Result<(), Box<dyn Error>> {
//!     let (mut ctx, mut handle) = Context::new();
//!
//!     let ctx_task = tokio::spawn(async move {
//!         // Set up a connection using your async framework of choice. We will need a read end, which is
//!         // AsyncRead, and write end, which is AsyncWrite, so we split the TcpStream
//!         // into ReadHalf and WriteHalf pair.
//!         let (rx, tx) = TcpStream::connect("127.0.0.1:1883").await?.into_split();
//!
//!         // Pass (ReadHalf, WriteHalf) pair into the context and connect with the broker on
//!         // the protocol level.
//!         ctx.set_up((rx.compat(), tx.compat_write())).connect(ConnectOpts::default()).await?;
//!
//!         // Awaiting the Context::run invocation will block the current task.
//!         if let Err(err) = ctx.run().await {
//!             eprintln!("[context] Error occured: \"{}\", exiting...", err);
//!         } else {
//!             println!("[context] Context exited.");
//!         }
//!
//!          Ok::<(), Box<dyn Error + Send + Sync>>(())
//!     });
//!
//!     /* ... */
//!
//!     ctx_task.await?;
//!     Ok(())
//! }
//! ```
//!
//! At this point, our [context](crate::Context) is up and running.
//!
//! Let's break down the above example.
//! `poster-rs` is a runtime agnostic library, which means that all the asynchronous operations are abstracted
//! using traits from the `futures-rs` crate. The result of this approach is that connection with the broker
//! must be established manually and the library only cares about receving ([AsyncRead](https://docs.rs/futures/latest/futures/io/trait.AsyncRead.html),
//! [AsyncWrite](https://docs.rs/futures/latest/futures/io/trait.AsyncWrite.html)) pair during the context creation.
//! This pair is usually obtained using some sort of `split` functions on streams/sockets in the networking libraries.
//! (See [tokio](https://docs.rs/tokio/latest/tokio/net/struct.TcpStream.html#method.into_split), [smol](https://docs.rs/smol/latest/smol/io/fn.split.html))
//!
//! [new](Context::new) factory method gives us ([Context], [ContextHandle]) tuple. [Context] is responsible
//! for handling the traffic between the client and the server. [ContextHandle] however, is a [cloneable](Clone) handle
//! to the [Context] actor and is used to perform all the MQTT operations.
//!
//! Method [run](crate::Context::run) blocks the task (on .await) until one of the following conditions is met:
//! 1. Graceful disconnection is performed (using [ContextHandle::disconnect](crate::ContextHandle::disconnect) method).
//!    The result is then ().
//! 2. Error occurs, resulting in [MqttError](crate::error::MqttError). This may be the result of socket closing,
//!    receiving DISCONNECT from the server, etc.
//!
//! ## Publishing
//!
//! Publishing is performed via the [publish](crate::ContextHandle::publish) method.
//!
//! ```no_run
//! # use std::error::Error;
//! # use poster::{prelude::*, *};
//! # use tokio::net::TcpStream;
//! # use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};
//! #
//! # #[tokio::main(flavor = "current_thread")]
//! # async fn main() -> Result<(), Box<dyn Error>> {
//! #   let (mut ctx, mut handle) = Context::new();
//! #   let ctx_task = tokio::spawn(async move {
//! #       let (rx, tx) = TcpStream::connect("127.0.0.1:1883").await?.into_split();
//! #       ctx.set_up((rx.compat(), tx.compat_write())).connect(ConnectOpts::default()).await?;
//! #       ctx.run().await?;
//! #       Ok::<(), Box<dyn Error + Send + Sync>>(())
//! #   });
//! #
//!     // ...
//!     let opts = PublishOpts::default().topic("topic").data("hello there".as_bytes());
//!     handle.publish(opts).await?;
//! #
//! #   ctx_task.await?;
//! #   Ok(())
//! # }
//! ```
//!
//! See [PublishOpts](crate::PublishOpts).
//!
//! ## Subscriptions
//!
//! Subscriptions are represented as async streams, obtained via the [stream](crate::SubscribeRsp::stream) method.
//! The general steps of subscribing are:
//! - await the invocation of [subscribe](crate::ContextHandle::subscribe) method
//! - validate the result (optionally)
//! - use [stream](crate::SubscribeRsp::stream) method in order to create a stream for
//! the subscription.
//!
//! Note that under the hood, the library uses subscription identifiers to group subscriptions.
//!
//! See [SubscribeOpts](crate::SubscribeOpts).
//!
//! ```no_run
//! # use std::{error::Error, str};
//! # use poster::{prelude::*, *};
//! # use tokio::net::TcpStream;
//! # use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};
//! #
//! # #[tokio::main(flavor = "current_thread")]
//! # async fn main() -> Result<(), Box<dyn Error>> {
//! #   let (mut ctx, mut handle) = Context::new();
//! #   let ctx_task = tokio::spawn(async move {
//! #       let (rx, tx) = TcpStream::connect("127.0.0.1:1883").await?.into_split();
//! #       ctx.set_up((rx.compat(), tx.compat_write())).connect(ConnectOpts::default()).await?;
//! #       ctx.run().await?;
//! #       Ok::<(), Box<dyn Error + Send + Sync>>(())
//! #   });
//! #
//!     // ...
//!     let opts = SubscribeOpts::default().subscription("topic", SubscriptionOptions::default());
//!     let rsp = handle.subscribe(opts).await?;
//!     let mut subscription = rsp.stream();
//!
//!     while let Some(msg) = subscription.next().await {
//!         println!("topic: {}; payload: {}", msg.topic_name(), str::from_utf8(msg.payload()).unwrap());
//!     }
//! #
//! #   ctx_task.await?;
//! #   Ok(())
//! # }
//! ```
//!
//! User may subscribe to multiple topics in one subscription request.
//!
//! ```no_run
//! # use std::{error::Error, str};
//! # use poster::{prelude::*, *};
//! # use tokio::net::TcpStream;
//! # use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};
//! #
//! # #[tokio::main(flavor = "current_thread")]
//! # async fn main() -> Result<(), Box<dyn Error>> {
//! #   let (mut ctx, mut handle) = Context::new();
//! #   let ctx_task = tokio::spawn(async move {
//! #       let (rx, tx) = TcpStream::connect("127.0.0.1:1883").await?.into_split();
//! #       ctx.set_up((rx.compat(), tx.compat_write())).connect(ConnectOpts::default()).await?;
//! #       ctx.run().await?;
//! #       Ok::<(), Box<dyn Error + Send + Sync>>(())
//! #   });
//! #
//!     // ...
//!     let opts = SubscribeOpts::default()
//!         .subscription("topic1", SubscriptionOptions::default())
//!         .subscription("topic2", SubscriptionOptions::default());
//!
//!     let mut subscription = handle.subscribe(opts).await?.stream();
//!
//!     while let Some(msg) = subscription.next().await {
//!         println!("topic: {}; payload: {}", msg.topic_name(), str::from_utf8(msg.payload()).unwrap());
//!     }
//! #
//! #   ctx_task.await?;
//! #   Ok(())
//! # }
//! ```
//!
//! Each subscription may be customized using the [SubscriptionOptions](crate::SubscriptionOptions).
//!
//! ```no_run
//! # use std::{error::Error, str};
//! # use poster::{prelude::*, *};
//! # use tokio::net::TcpStream;
//! # use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};
//! #
//! # #[tokio::main(flavor = "current_thread")]
//! # async fn main() -> Result<(), Box<dyn Error>> {
//! #   let (mut ctx, mut handle) = Context::new();
//! #   let ctx_task = tokio::spawn(async move {
//! #       let (rx, tx) = TcpStream::connect("127.0.0.1:1883").await?.into_split();
//! #       ctx.set_up((rx.compat(), tx.compat_write())).connect(ConnectOpts::default()).await?;
//! #       ctx.run().await?;
//! #       Ok::<(), Box<dyn Error + Send + Sync>>(())
//! #   });
//! #
//!     let opts = SubscribeOpts::default().subscription("topic", SubscriptionOptions {
//!         maximum_qos: QoS::AtLeastOnce,
//!         no_local: false,
//!         retain_as_published: true,
//!         retain_handling: RetainHandling::SendOnSubscribe,
//!     });
//! #
//! #   ctx_task.await?;
//! #   Ok(())
//! # }
//! ```
//!
//! [SubscribeRsp](crate::SubscribeRsp) struct represents the result of the subscription request. In order to access
//! per-topic reason codes, [payload](crate::SubscribeRsp::payload) method is used:
//!
//! ```no_run
//! # use std::{error::Error, str};
//! # use poster::{prelude::*, reason::SubackReason, *};
//! # use tokio::net::TcpStream;
//! # use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};
//! #
//! # #[tokio::main(flavor = "current_thread")]
//! # async fn main() -> Result<(), Box<dyn Error>> {
//! #   let (mut ctx, mut handle) = Context::new();
//! #   let ctx_task = tokio::spawn(async move {
//! #       let (rx, tx) = TcpStream::connect("127.0.0.1:1883").await?.into_split();
//! #       ctx.set_up((rx.compat(), tx.compat_write())).connect(ConnectOpts::default()).await?;
//! #       ctx.run().await?;
//! #       Ok::<(), Box<dyn Error + Send + Sync>>(())
//! #   });
//! #
//! #   let opts = SubscribeOpts::default();
//!     // ...
//!     let rsp = handle.subscribe(opts).await?;
//!     let all_ok = rsp.payload().iter().copied().all(|reason| reason == SubackReason::GranteedQoS2);
//! #
//! #   ctx_task.await?;
//! #   Ok(())
//! # }
//! ```
//!
//! ## Unsubscribing
//!
//! Unsubscribing is performed by the [unsubscribe](crate::ContextHandle::unsubscribe) method.
//! Note that it does NOT close the subscription stream (it could lead to logic errors).
//!
//! ```no_run
//! # use std::{error::Error, str};
//! # use poster::{prelude::*, *};
//! # use tokio::net::TcpStream;
//! # use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};
//! #
//! # #[tokio::main(flavor = "current_thread")]
//! # async fn main() -> Result<(), Box<dyn Error>> {
//! #   let (mut ctx, mut handle) = Context::new();
//! #   let ctx_task = tokio::spawn(async move {
//! #       let (rx, tx) = TcpStream::connect("127.0.0.1:1883").await?.into_split();
//! #       ctx.set_up((rx.compat(), tx.compat_write())).connect(ConnectOpts::default()).await?;
//! #       ctx.run().await?;
//! #       Ok::<(), Box<dyn Error + Send + Sync>>(())
//! #   });
//!     // ...
//!     let opts = UnsubscribeOpts::default().topic("topic");
//!     let rsp = handle.unsubscribe(opts).await?;
//! #
//! #   ctx_task.await?;
//! #   Ok(())
//! # }
//! ```
//!
//! As with subscribing, per topic reason codes can be obtained by the [payload](crate::UnsubscribeRsp::payload) method.
//!
//! See [UnsubscribeOpts](crate::UnsubscribeOpts).
//!
//! ## Keep alive and ping
//!
//! If the [keep_alive](crate::ConnectOpts::keep_alive) interval is set during the connection request,
//! the user must use the [ping](crate::ContextHandle::ping) method periodically.
//!
//! ## Disconnection
//!
//! Disconnection may be initiated either by user or the broker. When initiated by the broker, the [run](crate::Context::run) method
//! returns [Disconnected](crate::error::Disconnected) error.
//!
//! Graceful disconnection may be also performed by the user by using [disconnect](crate::ContextHandle::disconnect) method.
//! When disconnection is finished, [run](crate::Context::run) method returns ().
//!
//! ```no_run
//! # use std::{error::Error, str};
//! # use poster::{prelude::*, *};
//! # use tokio::net::TcpStream;
//! # use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};
//! #
//! # #[tokio::main(flavor = "current_thread")]
//! # async fn main() -> Result<(), Box<dyn Error>> {
//! #   let (mut ctx, mut handle) = Context::new();
//! #   let ctx_task = tokio::spawn(async move {
//! #       let (rx, tx) = TcpStream::connect("127.0.0.1:1883").await?.into_split();
//! #       ctx.set_up((rx.compat(), tx.compat_write())).connect(ConnectOpts::default()).await?;
//! #       ctx.run().await?;
//! #       Ok::<(), Box<dyn Error + Send + Sync>>(())
//! #   });
//! #
//!     // ...
//!     handle.disconnect(DisconnectOpts::default()).await?;
//! #
//! #   ctx_task.await?;
//! #   Ok(())
//! # }
//! ```
//!
//! See [DisconnectOpts](crate::DisconnectOpts).
//!
//! ## Error handling
//!
//! The main library error type is [MqttError](crate::error::MqttError) enum found in [error] module.
//!
//! ## TSL/SSL
//!
//! TSL/SSL libraries are available out there with AsyncRead, AsyncWrite TSL/SSL streams. These may be
//! supplied to the [set_up](crate::Context::set_up) method. The library does not handle encription on its own.
//!

mod client;
mod codec;
mod core;
mod io;

pub use crate::client::*;
pub use crate::codec::{RetainHandling, SubscriptionOptions};
pub use crate::core::{QoS, UserProperties};

/// Reason codes for different operations.
///
pub mod reason {
    pub use crate::codec::{
        AuthReason, ConnectReason, DisconnectReason, PubackReason, PubcompReason, PubrecReason,
        PubrelReason, SubackReason, UnsubackReason,
    };
}

/// Library error types.
///
pub mod error {
    pub use crate::client::error::*;
    pub use crate::core::error::*;
}

/// Reexports.
///
pub mod prelude {
    pub use either::Either;
    pub use futures::stream::{Stream, StreamExt};
}
