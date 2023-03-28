use clap::{arg, command, Parser};
use poster::{error::MqttError, prelude::*, ConnectOpts, Context, SubscribeOpts, SubscriptionOpts};
use smol::{io, net};
use std::{error::Error, str};

/// poster-rs subscription example using smol
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Broker IP
    #[arg(long)]
    host: String,

    /// Broker port
    #[arg(long, default_value_t = 1883)]
    port: u16,

    /// Topic
    #[arg(long)]
    topic: String,

    /// Username
    #[arg(long)]
    username: Option<String>,

    /// Password
    #[arg(long)]
    password: Option<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    smol::block_on(async {
        let (mut context, mut client) = Context::new();

        let subscription_task = smol::spawn(async move {
            // Set subscription parameters
            let opts = SubscribeOpts::new().subscription(&args.topic, SubscriptionOpts::default());

            // Send subscription request to the broker
            let mut subscription = client.subscribe(opts).await?.stream();

            // Asynchronously iterate over messages published to the subscribed topic
            while let Some(msg) = subscription.next().await {
                println!(
                    "[{}] {}",
                    msg.topic_name(),
                    str::from_utf8(msg.payload()).unwrap_or("<invalid UTF8 string>")
                );
            }

            Ok::<(), MqttError>(())
        });

        let stream = net::TcpStream::connect(format!("{}:{}", args.host, args.port)).await?;
        let (rx, tx) = io::split(stream);

        let mut opts = ConnectOpts::new();

        if args.username.is_some() {
            opts = opts.username(args.username.as_ref().unwrap());
        }

        if args.password.is_some() {
            opts = opts.password(args.password.as_ref().unwrap().as_bytes());
        }

        context.set_up((rx, tx)).connect(opts).await?;
        context.run().await?;

        subscription_task.await?;
        Ok(())
    })
}
