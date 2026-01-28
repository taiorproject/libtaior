use crate::cover::CoverTrafficGenerator;
use crate::discovery::NodeDiscovery;
use crate::identity::EphemeralIdentity;
use crate::modes::{ModeConfig, RoutingMode};
use crate::packet::TaiorPacket;
use crate::routing::Router;

pub use crate::modes::RoutingMode as Mode;

#[derive(Debug, Clone)]
pub struct SendOptions {
    pub mode: RoutingMode,
    pub hops: Option<u8>,
}

impl SendOptions {
    pub fn fast() -> Self {
        Self {
            mode: RoutingMode::Fast,
            hops: Some(1),
        }
    }

    pub fn mix() -> Self {
        Self {
            mode: RoutingMode::Mix,
            hops: Some(4),
        }
    }

    pub fn adaptive() -> Self {
        Self {
            mode: RoutingMode::Adaptive,
            hops: Some(2),
        }
    }

    pub fn custom(mode: RoutingMode, hops: u8) -> Self {
        Self {
            mode,
            hops: Some(hops),
        }
    }
}

impl Default for SendOptions {
    fn default() -> Self {
        Self::adaptive()
    }
}

pub struct Taior {
    identity: EphemeralIdentity,
    router: Router,
    discovery: NodeDiscovery,
    cover_generator: CoverTrafficGenerator,
}

impl Taior {
    pub fn new() -> Self {
        Self {
            identity: EphemeralIdentity::new(),
            router: Router::new(),
            discovery: NodeDiscovery::new(),
            cover_generator: CoverTrafficGenerator::default(),
        }
    }

    pub fn with_bootstrap(bootstrap: Vec<String>) -> Self {
        Self {
            identity: EphemeralIdentity::new(),
            router: Router::new(),
            discovery: NodeDiscovery::with_bootstrap(bootstrap),
            cover_generator: CoverTrafficGenerator::default(),
        }
    }

    pub fn address(&self) -> &str {
        self.identity.address.as_str()
    }

    pub fn add_node(&mut self, node: String) {
        self.discovery.add_node(node);
    }

    pub fn send(&mut self, data: &[u8], options: SendOptions) -> Result<TaiorPacket, String> {
        let config = match options.mode {
            RoutingMode::Fast => ModeConfig::fast(),
            RoutingMode::Mix => ModeConfig::mix(),
            RoutingMode::Adaptive => ModeConfig::adaptive(),
        };

        let config = if let Some(hops) = options.hops {
            config.with_hops(hops)
        } else {
            config
        };

        if self.cover_generator.should_send_cover() {
            let _ = self.cover_generator.generate_cover_packet(config.padding_size, config.hops)?;
        }

        let neighbors = self.discovery.get_neighbors();
        let next_hop = self.router.decide_next_hop(neighbors, &config);

        if next_hop.is_none() && self.discovery.count() == 0 {
            return Err("No neighbors available".to_string());
        }

        let packet = TaiorPacket::new(data, config.hops, config.padding_size, false)?;

        Ok(packet)
    }

    pub fn enable_cover_traffic(&mut self, enabled: bool, ratio: f32) {
        self.cover_generator = CoverTrafficGenerator::new(enabled, ratio);
    }
}

impl Default for Taior {
    fn default() -> Self {
        Self::new()
    }
}
