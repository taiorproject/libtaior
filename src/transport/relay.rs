use super::{TransportError, TransportResult};
use crate::identity::TaiorAddress;
use crate::packet::TaiorPacket;
use blake3::Hasher;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayAuth {
    pub token: String,
    pub expires_at: u64,
}

impl RelayAuth {
    pub fn generate(address: &TaiorAddress) -> Self {
        let mut hasher = Hasher::new();
        hasher.update(address.as_str().as_bytes());
        hasher.update(&std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_le_bytes());
        
        let token = hex::encode(hasher.finalize().as_bytes());
        let expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() + 3600;

        Self { token, expires_at }
    }

    pub fn is_valid(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now < self.expires_at
    }
}

pub struct RelayClient {
    relay_addr: SocketAddr,
    socket: Arc<UdpSocket>,
    auth: RelayAuth,
}

impl RelayClient {
    pub async fn new(relay_addr: SocketAddr, auth: RelayAuth) -> TransportResult<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0").await
            .map_err(|e| TransportError::RelayError(format!("bind: {}", e)))?;

        Ok(Self {
            relay_addr,
            socket: Arc::new(socket),
            auth,
        })
    }

    pub async fn send_via_relay(&self, packet: &TaiorPacket, dest: &TaiorAddress) -> TransportResult<()> {
        if !self.auth.is_valid() {
            return Err(TransportError::RelayError("auth expired".to_string()));
        }

        let envelope = RelayEnvelope {
            auth_token: self.auth.token.clone(),
            destination: dest.as_str().to_string(),
            packet: packet.clone(),
        };

        let serialized = serde_json::to_vec(&envelope)
            .map_err(|e| TransportError::RelayError(format!("serialize: {}", e)))?;

        self.socket.send_to(&serialized, self.relay_addr).await
            .map_err(|e| TransportError::RelayError(format!("send: {}", e)))?;

        Ok(())
    }

    pub async fn receive_from_relay(&self) -> TransportResult<(TaiorPacket, String)> {
        let mut buf = vec![0u8; 65536];
        let (len, _) = self.socket.recv_from(&mut buf).await
            .map_err(|e| TransportError::RelayError(format!("recv: {}", e)))?;

        let envelope: RelayEnvelope = serde_json::from_slice(&buf[..len])
            .map_err(|e| TransportError::RelayError(format!("deserialize: {}", e)))?;

        Ok((envelope.packet, envelope.destination))
    }
}

pub struct RelayServer {
    bind_addr: SocketAddr,
    socket: Arc<UdpSocket>,
    clients: Arc<RwLock<HashMap<String, SocketAddr>>>,
}

impl RelayServer {
    pub async fn new(bind_addr: SocketAddr) -> TransportResult<Self> {
        let socket = UdpSocket::bind(bind_addr).await
            .map_err(|e| TransportError::RelayError(format!("bind: {}", e)))?;

        Ok(Self {
            bind_addr,
            socket: Arc::new(socket),
            clients: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn run(&self) -> TransportResult<()> {
        let mut buf = vec![0u8; 65536];

        loop {
            let (len, src_addr) = self.socket.recv_from(&mut buf).await
                .map_err(|e| TransportError::RelayError(format!("recv: {}", e)))?;

            let envelope: RelayEnvelope = match serde_json::from_slice(&buf[..len]) {
                Ok(e) => e,
                Err(_) => continue,
            };

            {
                let mut clients = self.clients.write().await;
                clients.insert(envelope.auth_token.clone(), src_addr);
            }

            let clients = self.clients.read().await;
            if let Some(dest_addr) = clients.get(&envelope.destination) {
                let serialized = serde_json::to_vec(&envelope)
                    .map_err(|e| TransportError::RelayError(format!("serialize: {}", e)))?;

                let _ = self.socket.send_to(&serialized, dest_addr).await;
            }
        }
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.bind_addr
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RelayEnvelope {
    auth_token: String,
    destination: String,
    packet: TaiorPacket,
}
