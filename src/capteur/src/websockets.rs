//!
//! AMQP consumer that passes through data to websocket connections.
//!

use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use color_eyre::Report;
use futures_util::{SinkExt, StreamExt};
use lapin::{
    options::{BasicAckOptions, BasicConsumeOptions, QueueBindOptions, QueueDeclareOptions},
    types::FieldTable,
    Channel,
};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{
        broadcast::{self, Sender},
        mpsc::{unbounded_channel, UnboundedReceiver},
        Mutex,
    },
};
use tokio_tungstenite::{accept_async, tungstenite::Error, tungstenite::Message};
use tracing::{error, info};

///
/// A map of clients and how to communicate with them.
///
type ClientMap = HashMap<SocketAddr, Sender<String>>;

///
/// Broadcast incoming data to websockets
///
pub struct WebSocketPassthrough {
    channel: Arc<Mutex<Channel>>,
    clients: Arc<Mutex<ClientMap>>,
}

impl WebSocketPassthrough {
    ///
    /// Create a [`WebSocketPassthrough`] from an existing channel.
    ///
    pub fn from_channel(channel: Arc<Mutex<Channel>>) -> Self {
        Self {
            channel,
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    ///
    /// Read from AMQP and emit to WebSocket clients.
    ///
    pub async fn stream(&mut self) -> Result<(), Report> {
        // Get our channel and lock it
        let channel = self.channel.lock().await;

        // Create a queue to consume from
        let _ = channel
            .queue_declare(
                "websocket-passthrough",
                QueueDeclareOptions::default(),
                FieldTable::default(),
            )
            .await?;

        // Bind the exchange to our queue
        let _ = channel
            .queue_bind(
                "websocket-passthrough",
                "capteur.fanout",
                "",
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await?;

        // Create a simple consumer
        let mut consumer = channel
            .basic_consume(
                "websocket-passthrough",
                "websocket-passthrough-consumer",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await?;

        info!("Start websocket stream");

        // Create and wrap our inbound channel
        let (tx, rx) = unbounded_channel::<String>();
        let inbound = Arc::new(Mutex::new(rx));

        let consumer_stream = tokio::task::spawn(async move {
            info!("DatabaseBackfill consumer started");
            while let Some(delivery) = consumer.next().await {
                let delivery = delivery.expect("Error in consumer");
                let data = std::str::from_utf8(&delivery.data).expect("Failed to parse data");
                info!("WS Received: {}", &data);
                tx.send(data.to_string()).unwrap();
                delivery
                    .ack(BasicAckOptions::default())
                    .await
                    .expect("Couldn't ACK received message");
            }
        });

        // Drop our channel lock and stream...
        drop(channel);
        tokio::select! {
            _ = consumer_stream => (),
            _ = Self::listen(self.clients.clone(), inbound.clone()) => (),
        };

        Ok(())
    }

    async fn listen(
        clients: Arc<Mutex<ClientMap>>,
        inbound: Arc<Mutex<UnboundedReceiver<String>>>,
    ) {
        // Listen to the specified addr for TCP/IP connections
        let addr = "localhost:9002";
        let listener = TcpListener::bind(&addr)
            .await
            .expect(format!("Can't listen at addr: {}", addr).as_str());
        info!("Listening on: {}", addr);

        // Accept loop
        while let Ok((stream, _)) = listener.accept().await {
            let peer = stream
                .peer_addr()
                .expect("Connected streams should have a peer address");
            info!("Peer address: {}", peer);

            tokio::spawn(Self::accept_connection(
                peer,
                stream,
                clients.clone(),
                inbound.clone(),
            ));
        }
    }

    async fn accept_connection(
        peer: SocketAddr,
        stream: TcpStream,
        clients: Arc<Mutex<ClientMap>>,
        inbound: Arc<Mutex<UnboundedReceiver<String>>>,
    ) {
        // Handle broken connections
        match Self::handle_connection(peer, stream, clients.clone(), inbound).await {
            Err(e) => match e {
                Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8 => (),
                err => {
                    error!("Error processing connection: {}", err);
                }
            },
            _ => (),
        }
        // Remove the peer since the connection is dead, this will do nothing if we never made it [`handle_connect`]
        let mut connections = clients.lock().await;
        connections.remove(&peer);
    }

    async fn handle_connection(
        peer: SocketAddr,
        stream: TcpStream,
        clients: Arc<Mutex<ClientMap>>,
        inbound: Arc<Mutex<UnboundedReceiver<String>>>,
    ) -> tungstenite::Result<()> {
        // Upgrade the TCP/HTTP connection to a WebSocket
        let mut ws_stream = accept_async(stream).await.expect("Failed to accept");

        info!("New WebSocket connection: {}", peer);

        // Add our client to the ClientMap
        {
            let mut connections = clients.lock().await;
            let (tx, _) = broadcast::channel::<String>(512); // TODO: This could be tuned

            connections.insert(peer.clone(), tx);
        }

        while let Some(msg) = inbound.lock().await.recv().await {
            ws_stream.send(Message::from(msg)).await?;
        }

        // // Echo... echo... echo...
        // while let Some(msg) = ws_stream.next().await {
        //     let msg = msg?;
        //     if msg.is_text() || msg.is_binary() {
        //         ws_stream.send(msg).await?;
        //     }
        // }

        Ok(())
    }
}
