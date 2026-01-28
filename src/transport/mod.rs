pub mod quic;
pub mod nat;
pub mod relay;

pub use quic::{QuicTransport, QuicConfig};
pub use nat::NatTraversal;
pub use relay::{RelayClient, RelayServer, RelayAuth};

use crate::packet::TaiorPacket;
use std::net::SocketAddr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TransportError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Send failed: {0}")]
    SendFailed(String),
    #[error("Receive failed: {0}")]
    ReceiveFailed(String),
    #[error("NAT traversal failed: {0}")]
    NatTraversalFailed(String),
    #[error("Relay error: {0}")]
    RelayError(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type TransportResult<T> = Result<T, TransportError>;

#[async_trait::async_trait]
pub trait Transport: Send + Sync {
    async fn send(&mut self, packet: &TaiorPacket, dest: SocketAddr) -> TransportResult<()>;
    async fn receive(&mut self) -> TransportResult<(TaiorPacket, SocketAddr)>;
    async fn connect(&mut self, addr: SocketAddr) -> TransportResult<()>;
    async fn close(&mut self) -> TransportResult<()>;
}
