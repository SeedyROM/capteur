//!
//! Transport layers, only AMQP is currently supported
//!

use std::time::Duration;

use color_eyre::Report;
use lapin::{
    options::{BasicPublishOptions, QueueDeclareOptions},
    types::FieldTable,
    BasicProperties, Connection, ConnectionProperties,
};
use rand::Rng;
use tracing::{error, info};

///
/// Dummy stream to push to RabbitMQ, **REMOVE ME**
///
pub async fn stream_to_amqp() -> Result<(), Report> {
    // Create the RabbitMQ connection
    let addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://127.0.0.1:5672/%2f".into());
    let conn = Connection::connect(&addr, ConnectionProperties::default()).await?;
    let channel = conn.create_channel().await?;

    // Create a test queue
    let _ = channel
        .queue_declare(
            "fake-data",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;

    // Emit our fake messages
    loop {
        // Mock random sensor data
        let data: f64 = rand::thread_rng().gen();

        info!("Publishing data to RabbitMQ...");
        let result = channel
            .basic_publish(
                "",
                "fake-data",
                BasicPublishOptions::default(),
                data.to_string().as_bytes(),
                BasicProperties::default(),
            )
            .await;

        if let Err(error) = result {
            error!(?error)
        }

        tokio::time::sleep(Duration::from_millis(500)).await
    }
}
