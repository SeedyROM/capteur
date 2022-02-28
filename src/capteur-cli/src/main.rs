//!
//! The command line interface to emit messages coming from the serial/UART update ticks.
//!
//! **Supported transport layers:**
//!
//! - [tokio-amqp](https://crates.io/crates/tokio-amqp)
//! - *TBD: Other transport layers yet TBD!*
//!

use color_eyre::Report;

use capteur::{
    setup_environment,
    transports::{Transport, AMQP},
    websockets::WebSocketPassthrough,
};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Report> {
    setup_environment()?;

    info!("Client is spinning up...");
    info!("Let's emit some garbage!!!");

    let mut transport = AMQP::new().await?;
    let mut ws_passthrough = WebSocketPassthrough::from_channel(transport.channel.clone());

    let amqp_transport = tokio::task::spawn(async move { transport.stream().await });
    let amqp_passthrough = tokio::task::spawn(async move { ws_passthrough.stream().await });

    tokio::select! {
        _ = amqp_transport => (),
        _ = amqp_passthrough => (),
    };

    info!("Shutting down...");

    Ok(())
}
