//!
//! Transport layers, only AMQP is currently supported
//!

use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use color_eyre::Report;
use lapin::{
    options::{BasicPublishOptions, QueueDeclareOptions},
    types::FieldTable,
    BasicProperties, Channel, Connection, ConnectionProperties,
};
use rand::Rng;
use tokio::sync::Mutex;
use tracing::{error, info};

///
/// Trait to handle all transport layers.
///
#[async_trait]
pub trait Transport<T> {
    ///
    /// Create a new transport, just do AMQP for now.
    ///
    async fn new() -> Result<T, Report>;

    ///
    /// Stream data into the transport.
    ///
    async fn stream(&mut self) -> Result<(), Report>;
}

///
/// Simple AMQP transport
///
pub struct AMQP {
    channel: Arc<Mutex<Channel>>,
}

#[async_trait]
impl Transport<Self> for AMQP {
    ///
    /// Create a new AMQP transport layer
    ///
    async fn new() -> Result<Self, Report> {
        // Create the RabbitMQ connection
        let addr =
            std::env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://127.0.0.1:5672/%2f".into());
        let conn = Connection::connect(&addr, ConnectionProperties::default()).await?;
        let channel = conn.create_channel().await?;
        Ok(AMQP {
            channel: Arc::new(Mutex::new(channel)),
        })
    }

    ///
    /// Dummy stream to push to RabbitMQ, **REMOVE ME**
    ///
    async fn stream(&mut self) -> Result<(), Report> {
        let channel = self.channel.lock().await;

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
}
