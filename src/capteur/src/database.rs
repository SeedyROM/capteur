use std::sync::Arc;

use async_trait::async_trait;
use color_eyre::Report;
use futures::StreamExt;
use lapin::{
    options::{BasicAckOptions, BasicConsumeOptions, QueueBindOptions, QueueDeclareOptions},
    types::FieldTable,
    Channel,
};
use mongodb::{options::ClientOptions, Client, Database};
use tokio::sync::Mutex;
use tracing::info;

use crate::util::Consumer;

pub struct DatabaseConsumer {
    channel: Arc<Mutex<Channel>>,
    client: Option<Arc<Mutex<Client>>>,
    database: Option<Arc<Mutex<Database>>>,
}

impl DatabaseConsumer {
    pub fn from_channel(channel: Arc<Mutex<Channel>>) -> Self {
        DatabaseConsumer {
            channel,
            client: None,
            database: None,
        }
    }
}

#[async_trait]
impl Consumer for DatabaseConsumer {
    async fn stream(&mut self) -> Result<(), Report> {
        // Configure the the mongodb connection
        let addr =
            std::env::var("MONGODB_ADDR").unwrap_or_else(|_| "mongodb://localhost:27017".into());
        let db_name = std::env::var("MONGODB_DB").unwrap_or_else(|_| "capteur".into());
        let mut client_options = ClientOptions::parse(&addr).await?;
        client_options.app_name = Some(db_name.to_string());

        // Initialize the client and db connection
        let client = Client::with_options(client_options)?;
        let database = client.database("fake-data-backup");
        self.client = Some(Arc::new(Mutex::new(client)));
        self.database = Some(Arc::new(Mutex::new(database)));

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
                "fake-data",
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

        // Stream data into the database
        info!("DatabaseBackfill consumer started...");
        while let Some(delivery) = consumer.next().await {
            let delivery = delivery.expect("Error in consumer");
            let data = std::str::from_utf8(&delivery.data).expect("Failed to parse data");
            info!("DB Received: {}", &data);

            // TODO: Insert into the DB

            delivery
                .ack(BasicAckOptions::default())
                .await
                .expect("Couldn't ACK received message");
        }

        Ok(())
    }
}
