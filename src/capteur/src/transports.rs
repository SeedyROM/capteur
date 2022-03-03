//!
//! Transport layers, only AMQP is currently supported
//!

use std::{collections::BTreeMap, sync::Arc, time::Duration};

use async_trait::async_trait;
use color_eyre::Report;
use lapin::{
    options::{BasicPublishOptions, ExchangeDeclareOptions},
    types::FieldTable,
    BasicProperties, Channel, Connection, ConnectionProperties, ExchangeKind,
};
use rand::Rng;
use tokio::sync::Mutex;
use tracing::{error, info};

use crate::{
    messages::{Message, SensorReading},
    util::get_epoch_ms,
};

///
/// Supported transports.
///
pub enum TransportType {
    AMQP,
    MQTT,
    Kafka,
    Redis,
}

///
/// Trait to handle all transport layers.
///
#[async_trait]
pub trait Transport {
    ///
    /// Stream data into the transport.
    ///
    async fn stream(&mut self) -> Result<(), Report>;
}

///
/// Simple AMQP transport
///
pub struct AMQP {
    pub channel: Arc<Mutex<Channel>>,
}

impl AMQP {
    ///
    /// Create a new AMQP transport layer
    ///
    pub async fn new() -> Result<Self, Report> {
        // Create the RabbitMQ connection
        let addr =
            std::env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://127.0.0.1:5672/%2f".into());
        let conn = Connection::connect(&addr, ConnectionProperties::default()).await?;
        let channel = conn.create_channel().await?;

        Ok(AMQP {
            channel: Arc::new(Mutex::new(channel)),
        })
    }
}

#[async_trait]
impl Transport for AMQP {
    ///
    /// Dummy stream to push to RabbitMQ, **REMOVE ME**
    ///
    async fn stream(&mut self) -> Result<(), Report> {
        // Create a test queue
        {
            let channel = self.channel.lock().await;

            channel
                .exchange_declare(
                    "capteur.fanout",
                    ExchangeKind::Fanout,
                    ExchangeDeclareOptions::default(),
                    FieldTable::default(),
                )
                .await?;
        }

        // Emit our fake messages
        loop {
            let channel = self.channel.lock().await;

            // Mock random sensor data
            let mut sensors = BTreeMap::new();
            sensors.insert(
                "Barometer".to_string(),
                SensorReading::Measurement {
                    value: rand::thread_rng().gen::<f64>() * 500f64,
                    unit: "Pa".to_string(),
                },
            );
            sensors.insert(
                "Valve 1".to_string(),
                SensorReading::Boolean {
                    value: if rand::thread_rng().gen::<f64>() > 0.5 {
                        true
                    } else {
                        false
                    },
                },
            );
            let message = Message::Reading {
                timestamp: get_epoch_ms(),
                sensors,
            };

            // Publish to the the queue
            info!("Publishing data to RabbitMQ");
            let result = channel
                .basic_publish(
                    "capteur.fanout",
                    "",
                    BasicPublishOptions::default(),
                    serde_json::to_string(&message).unwrap().as_bytes(),
                    BasicProperties::default(),
                )
                .await;

            if let Err(error) = result {
                error!(?error)
            }

            tokio::time::sleep(Duration::from_millis(400)).await
        }
    }
}
