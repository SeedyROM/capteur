//!
//! The command line interface to emit messages coming from the serial/UART update ticks.
//!
//! **Supported transport layers:**
//!
//! - [amqp](https://crates.io/crates/lapin)
//! - *TBD: Other transport layers yet TBD!*
//!

use capteur::{
    database::DatabaseConsumer,
    setup_environment,
    transports::{Transport, AMQP},
    util::Consumer,
    websockets::WebSocketPassthrough,
};
use color_eyre::Report;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Report> {
    setup_environment()?;

    info!("Starting capteur-cli");

    // Create our processes
    let mut transport = AMQP::new().await?;
    let mut ws_passthrough = WebSocketPassthrough::from_channel(transport.channel.clone());
    let mut database_consumer = DatabaseConsumer::from_channel(transport.channel.clone());

    // Spawn the tasks
    let amqp_transport = tokio::task::spawn(async move { transport.stream().await });
    let amqp_passthrough = tokio::task::spawn(async move { ws_passthrough.stream().await });
    let amqp_database_consumer =
        tokio::task::spawn(async move { database_consumer.stream().await });

    // Wait for one task to die...
    tokio::select! {
        _ = amqp_transport => (),
        _ = amqp_passthrough => (),
        _ = amqp_database_consumer => (),
    };

    info!("Shutting down capteur-cli");

    Ok(())
}
