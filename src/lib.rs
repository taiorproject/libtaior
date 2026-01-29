pub mod identity;
pub mod modes;
pub mod packet;
pub mod routing;
pub mod cover;
pub mod discovery;
pub mod api;
#[cfg(not(target_arch = "wasm32"))]
pub mod transport;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

pub use api::{Taior, SendOptions};
pub use identity::TaiorAddress;
pub use modes::RoutingMode;
#[cfg(not(target_arch = "wasm32"))]
pub use transport::{QuicTransport, QuicConfig, NatTraversal, RelayClient, RelayServer, RelayAuth};
