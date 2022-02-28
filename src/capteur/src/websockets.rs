//!
//! AMQP consumer that passes through data to websocket connections.
//!

use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use futures_util::{SinkExt, StreamExt};
use lapin::Channel;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{
        broadcast::{self, Sender},
        Mutex,
    },
};
use tokio_tungstenite::{accept_async, tungstenite::Error};
use tracing::{error, info};

///
/// A map of clients and how to communicate with them.
///
type ClientMap = HashMap<SocketAddr, Sender<String>>;

///
/// Broadcast incoming data to websockets
///
pub struct WebSocketPassthrough {
    inbound: Arc<Mutex<Channel>>,
    clients: Arc<Mutex<ClientMap>>,
}

impl WebSocketPassthrough {
    pub fn from_channel(channel: Arc<Mutex<Channel>>) -> Self {
        Self {
            inbound: channel,
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn stream(&mut self) {
        let addr = "localhost:9002";
        let listener = TcpListener::bind(&addr)
            .await
            .expect(format!("Can't listen at addr: {}", addr).as_str());
        info!("Listening on: {}", addr);

        while let Ok((stream, _)) = listener.accept().await {
            let peer = stream
                .peer_addr()
                .expect("connected streams should have a peer address");
            info!("Peer address: {}", peer);

            tokio::spawn(Self::accept_connection(peer, stream, self.clients.clone()));
        }
    }

    async fn accept_connection(
        peer: SocketAddr,
        stream: TcpStream,
        clients: Arc<Mutex<ClientMap>>,
    ) {
        match Self::handle_connection(peer, stream, clients).await {
            Err(e) => match e {
                Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8 => (),
                err => error!("Error processing connection: {}", err),
            },
            _ => (),
        }
    }

    async fn handle_connection(
        peer: SocketAddr,
        stream: TcpStream,
        clients: Arc<Mutex<ClientMap>>,
    ) -> tungstenite::Result<()> {
        let mut ws_stream = accept_async(stream).await.expect("Failed to accept");

        info!("New WebSocket connection: {}", peer);

        // Add our client to the ClientMap
        {
            let mut connections = clients.lock().await;
            let (tx, _) = broadcast::channel::<String>(512); // TODO: This could be tuned

            connections.insert(peer.clone(), tx);
        }

        while let Some(msg) = ws_stream.next().await {
            let msg = msg?;
            if msg.is_text() || msg.is_binary() {
                ws_stream.send(msg).await?;
            }
        }

        Ok(())
    }
}
