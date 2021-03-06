//!
//! Database [`Consumer`] stream.
//!

use std::sync::Arc;

use async_trait::async_trait;
use color_eyre::Report;
use futures::StreamExt;
use lapin::{
    options::{BasicAckOptions, BasicConsumeOptions, QueueBindOptions, QueueDeclareOptions},
    types::FieldTable,
    Channel,
};
use mongodb::{bson::doc, options::ClientOptions, Client};
use tokio::sync::{mpsc, Mutex};
use tracing::info;

use crate::util::Consumer;

///
/// Consume data from AMQP and insert into MongoDB with backpressure.
///
pub struct DatabaseConsumer {
    channel: Arc<Mutex<Channel>>,
}

impl DatabaseConsumer {
    pub fn from_channel(channel: Arc<Mutex<Channel>>) -> Self {
        DatabaseConsumer { channel }
    }
}

#[async_trait]
impl Consumer for DatabaseConsumer {
    ///
    /// Stream database from a consumer into the database.
    ///
    async fn stream(&mut self) -> Result<(), Report> {
        // Configure the the mongodb connection
        let addr =
            std::env::var("MONGODB_ADDR").unwrap_or_else(|_| "mongodb://localhost:27017".into());
        let db_name = std::env::var("MONGODB_DB").unwrap_or_else(|_| "capteur".into());
        let mut client_options = ClientOptions::parse(&addr).await?;
        client_options.app_name = Some(db_name.to_string());

        // Initialize the client and db connection
        let client = Client::with_options(client_options)?;
        let database = client.database("fake-data-backup").collection("data");

        // Get our channel and lock it
        let channel = self.channel.lock().await;

        // Create a queue to consume from, this one is durable to handle downtime from the client
        let _ = channel
            .queue_declare(
                "database-backfill",
                QueueDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                FieldTable::default(),
            )
            .await?;

        // Bind the exchange to our queue
        let _ = channel
            .queue_bind(
                "database-backfill",
                "capteur.fanout",
                "",
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await?;

        // Create a simple consumer
        let mut consumer = channel
            .basic_consume(
                "database-backfill",
                "database-backfill-consumer",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await?;

        // Drop the lock and stream the incoming data
        drop(channel);

        // Use a bounded channel for backpressure, 256 is the max for now
        let (consumer_tx, mut consumer_rx) = mpsc::channel::<String>(256);

        let consumer_stream = tokio::task::spawn(async move {
            // Stream data into the database
            info!("DatabaseBackfill consumer started");
            while let Some(delivery) = consumer.next().await {
                let delivery = delivery.expect("Error in consumer");
                let data = std::str::from_utf8(&delivery.data).expect("Failed to parse data");

                info!("DB Received: {}", &data);
                consumer_tx
                    .send(data.to_string())
                    .await
                    .expect("Failed stream consumer message to database");

                delivery
                    .ack(BasicAckOptions::default())
                    .await
                    .expect("Couldn't ACK received message");
            }
        });

        let database_stream = tokio::task::spawn(async move {
            while let Some(message) = consumer_rx.recv().await {
                let data = message.as_str();
                let _ = database
                    .insert_one(doc! { data: data }, None)
                    .await
                    .expect("Failed to insert into the database");
            }
        });

        tokio::select! {
            _ = consumer_stream => (),
            _ = database_stream => (),
        }

        Ok(())
    }
}
