pub mod identity;
pub mod modes;
pub mod packet;
pub mod routing;
pub mod cover;
pub mod discovery;
pub mod api;
pub mod circuit;
pub mod cover_traffic;

#[cfg(not(target_arch = "wasm32"))]
pub mod transport;

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
pub mod wasm;

pub use api::{Taior, SendOptions};
pub use identity::TaiorAddress;
pub use modes::RoutingMode;
pub use circuit::{Circuit, CircuitBuilder, CircuitNode, OnionEncryptor};
pub use cover_traffic::{CoverTrafficConfig, CoverTrafficGenerator, AdaptiveCoverTraffic};

#[cfg(not(target_arch = "wasm32"))]
pub use transport::{QuicTransport, QuicConfig, NatTraversal, RelayClient, RelayServer, RelayAuth};
