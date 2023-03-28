use clap::{arg, command, Parser};
use poster::{error::MqttError, ConnectOpts, Context, DisconnectOpts, PublishOpts, QoS};
use std::{
    error::Error,
    str::{self},
};
use tokio::net;
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

fn make_qos(str: &str) -> QoS {
    match str {
        "0" => QoS::AtMostOnce,
        "1" => QoS::AtLeastOnce,
        "2" => QoS::ExactlyOnce,
        _ => panic!("Invalid QoS value, must be 0, 1 or 2."),
    }
}

/// poster-rs publish example using tokio
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

    /// Message
    #[arg(long)]
    message: String,

    /// Quality of Service
    #[arg(long, default_value_t = String::from("0"))]
    qos: String,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let args = Args::parse();

    let (mut context, mut client) = Context::new();

    let ctx_task = tokio::spawn(async move {
        let stream = net::TcpStream::connect(format!("{}:{}", args.host, args.port)).await?;
        let (rx, tx) = stream.into_split();

        let mut opts = ConnectOpts::new();

        if args.username.is_some() {
            opts = opts.username(args.username.as_ref().unwrap());
        }

        if args.password.is_some() {
            opts = opts.password(args.password.as_ref().unwrap().as_bytes());
        }

        context
            .set_up((rx.compat(), tx.compat_write()))
            .connect(opts)
            .await?;

        match context.run().await {
            Err(MqttError::SocketClosed(_)) => {}
            Err(err) => eprintln!("Error: \"{}\".", err),
            _ => {}
        }

        Ok::<(), Box<dyn Error + Send + Sync>>(())
    });

    client
        .publish(
            PublishOpts::new()
                .topic_name(&args.topic)
                .qos(make_qos(&args.qos))
                .payload(args.message.as_bytes()),
        )
        .await?;
    client.disconnect(DisconnectOpts::default()).await?;

    ctx_task.await?
}
