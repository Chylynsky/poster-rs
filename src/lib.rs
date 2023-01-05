#![forbid(unsafe_code, unreachable_pub)]
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
//! ```
//! use std::error::Error;
//! use poster::{
//!     prelude::*, ConnectOpts, Context, DisconnectOpts, PublishOpts, QoS, SubscribeOpts,
//!     SubscriptionOptions, UnsubscribeOpts,
//! };
//! use tokio::net::TcpStream;
//! use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};
//!
//! #[tokio::main(flavor = "current_thread")]
//! async fn main() -> Result<(), Box<dyn Error>> {
//!     // Set up a connection using your async framework of choice. We will need a read end, which is
//!     // AsyncRead, and write end, which is AsyncWrite, so we split the TcpStream
//!     // into ReadHalf and WriteHalf pair.
//!     let (rx, tx) = TcpStream::connect("127.0.0.1:1883").await?.into_split();
//!
//!     // Create a pair of MQTT client context and a handle.
//!     // Note that Tokio needs a compatibility layer to be used with futures.
//!     let (mut ctx, mut handle) = Context::new(rx.compat(), tx.compat_write());
//!
//!     // The last part of set-up boilerplate is spawning a task and starting the context there.
//!     let ctx_task = tokio::spawn(async move {
//!         if let Err(err) = ctx.run().await {
//!             eprintln!("[ctx_task] Error occured: \"{}\", exiting...", err);
//!         } else {
//!             println!("[ctx_task] Context exited.");
//!         }
//!     });
//! }
//! ```
//!
//! At this point, our [context](crate::Context) is up and running, however it is still not connected to the server on the MQTT protocol level.
//!
//! Let's break down the above example.
//! `poster-rs` is a runtime agnostic library, which means that all the asynchronous operations are abstracted
//! using traits from the `futures-rs` crate. The result of this approach is that connection with the broker
//! must be established manually and the library only cares about receving ([AsyncRead](https://docs.rs/futures/latest/futures/io/trait.AsyncRead.html),
//! [AsyncWrite](https://docs.rs/futures/latest/futures/io/trait.AsyncWrite.html)) pair during the context creation.
//! This pair is usually obtained using some sort of `split` functions on streams/sockets in the networking libraries.
//! (See [tokio](https://docs.rs/tokio/latest/tokio/net/struct.TcpStream.html#method.into_split), [smol](https://docs.rs/smol/latest/smol/io/fn.split.html))
//!
//! [new](Context::new) factory method gives us ([Context], [ContextHandle]) tuple. [ContextHandle] is responsible
//! for handling the traffic between the client and the server. [ContextHandle] however, is a [cloneable](Clone) handle
//! to the [Context] actor and is used to perform all the MQTT operations.
//!
//! Method [run](crate::Context::run) blocks the task (on .await) until one of the following conditions is met:
//! 1. Graceful disconnection is performed (using [ContextHandle::disconnect](crate::ContextHandle::disconnect) method).
//!    The result is then ().
//! 2. Error occurs, resulting in [MqttError](crate::error::MqttError). This may be the result of socket closing,
//!    receiving DISCONNECT from the server, etc.
//!
//! Having the above setup, we can continue connecting using the [connect](crate::ContextHandle::connect) method:
//!
//! ```
//! # use std::error::Error;
//! # use poster::{
//! #     prelude::*, ConnectOpts, Context, DisconnectOpts, PublishOpts, QoS, SubscribeOpts,
//! #     SubscriptionOptions, UnsubscribeOpts,
//! # };
//! # use tokio::net::TcpStream;
//! # use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};
//! #[tokio::main(flavor = "current_thread")]
//! async fn main() -> Result<(), Box<dyn Error>> {
//! # let (rx, tx) = TcpStream::connect("127.0.0.1:1883").await?.into_split();
//! # let (mut ctx, mut handle) = Context::new(rx.compat(), tx.compat_write());
//! # let ctx_task = tokio::spawn(async move {
//! #     if let Err(err) = ctx.run().await {
//! #         eprintln!("[ctx_task] Error occured: \"{}\", exiting...", err);
//! #     } else {
//! #         println!("[ctx_task] Context exited.");
//! #     }
//! # });
//!     /* ... */
//!
//!     let opts = ConnectOpts::new();
//!     ctx.connect(opts).await?;
//! }
//! ```
//!
//! Note that all of the MQTT operation results (represented by the [ContextHandle](crate::ContextHandle) async methods) and messages (being usually packets sent by the broker)
//! are obtained at the point of .await.
//!
//! ## Subscriptions
//!
//! ```
//! # use std::error::Error;
//! # use poster::{
//! #     prelude::*, ConnectOpts, Context, DisconnectOpts, PublishOpts, QoS, SubscribeOpts,
//! #     SubscriptionOptions, UnsubscribeOpts,
//! # };
//! # use tokio::net::TcpStream;
//! # use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};
//! #[tokio::main(flavor = "current_thread")]
//! async fn main() -> Result<(), Box<dyn Error>> {
//! # let (rx, tx) = TcpStream::connect("127.0.0.1:1883").await?.into_split();
//! # let (mut ctx, mut handle) = Context::new(rx.compat(), tx.compat_write());
//! # let ctx_task = tokio::spawn(async move {
//! #     if let Err(err) = ctx.run().await {
//! #         eprintln!("[ctx_task] Error occured: \"{}\", exiting...", err);
//! #     } else {
//! #         println!("[ctx_task] Context exited.");
//! #     }
//! # });
//!     /* ... */
//!
//!     let opts = SubscribeOpts::new();
//!     let subrsp = ctx.subscribe(opts.subscription("a/b", SubscriptionOptions::default())).await?;
//!
//!     // Validate the subscription response. Method payload accesses the list of SubackReason values.
//!     assert!(subrsp.payload().into_iter().all(|result| result == SubackReason::Success));
//!
//!     // Transform response into the async stream of messages published to the topics specified in opts.
//!     let mut subscription = subrsp.stream();
//!     
//!     if let Some(msg) = subscription.next().await {
//!         println!(
//!             "[{}] {}",
//!             msg.topic_name(),
//!             str::from_utf8(msg.payload()).unwrap()
//!         );
//!     }
//! }
//! ```
//!
//! ## Unsubscribing
//!
//! TODO
//!
//! ## Keep alive and ping
//!
//! TODO
//!
//! ## Disconnection
//!
//! TODO
//!
//! ## Error handling
//!
//! The main library error type is [MqttError](crate::error::MqttError) found in [error] module.
//!
//! ## TSL/SSL
//!
//! TODO
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

#[allow(missing_docs)]
pub mod prelude {
    pub use futures::stream::{Stream, StreamExt};
}
