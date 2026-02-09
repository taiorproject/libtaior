use crate::cover::CoverTrafficGenerator;
use crate::circuit::{Circuit, CircuitBuilder, OnionEncryptor, CircuitError};
use crate::discovery::NodeDiscovery;
use crate::identity::{EphemeralIdentity, TaiorAddress};
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
    active_circuit: Option<Circuit>,
    circuit_builder: CircuitBuilder,
}

impl Taior {
    pub fn new() -> Self {
        Self {
            identity: EphemeralIdentity::new(),
            router: Router::new(),
            discovery: NodeDiscovery::new(),
            cover_generator: CoverTrafficGenerator::default(),
            active_circuit: None,
            circuit_builder: CircuitBuilder::new(1, 5, 600),
        }
    }

    pub fn with_bootstrap(bootstrap: Vec<String>) -> Self {
        let mut instance = Self {
            identity: EphemeralIdentity::new(),
            router: Router::new(),
            discovery: NodeDiscovery::with_bootstrap(bootstrap.clone()),
            cover_generator: CoverTrafficGenerator::default(),
            active_circuit: None,
            circuit_builder: CircuitBuilder::new(1, 5, 600),
        };

        // Register bootstrap nodes in circuit builder
        for node in &bootstrap {
            let (_, addr) = TaiorAddress::generate();
            instance.circuit_builder.add_node(node.clone(), addr);
        }

        instance
    }

    pub fn address(&self) -> &str {
        self.identity.address.as_str()
    }

    pub fn add_node(&mut self, node: String) {
        let (_, addr) = TaiorAddress::generate();
        self.circuit_builder.add_node(node.clone(), addr);
        self.discovery.add_node(node);
    }

    fn ensure_circuit(&mut self, target_hops: usize) -> Result<(), String> {
        // Rebuild circuit if expired or not present
        let needs_rebuild = match &self.active_circuit {
            Some(circuit) => circuit.is_expired(),
            None => true,
        };

        if needs_rebuild {
            match self.circuit_builder.build_circuit(target_hops) {
                Ok(circuit) => {
                    self.active_circuit = Some(circuit);
                    Ok(())
                }
                Err(CircuitError::InsufficientNodes(_)) => {
                    // Not enough nodes for a circuit — allow direct send with packet encryption only
                    self.active_circuit = None;
                    Ok(())
                }
                Err(e) => Err(format!("Circuit build failed: {}", e)),
            }
        } else {
            Ok(())
        }
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
        let _next_hop = self.router.decide_next_hop(neighbors, &config);

        // Build/refresh circuit for onion encryption
        self.ensure_circuit(config.hops as usize)?;

        // Create base packet with padding
        let packet = TaiorPacket::new(data, config.hops, config.padding_size, false)?;

        // Apply onion encryption if circuit is available
        if let Some(circuit) = &self.active_circuit {
            let encryptor = OnionEncryptor::new(circuit.clone());
            let onion_encrypted = encryptor.encrypt_onion(&packet.encrypted_payload)
                .map_err(|e| format!("Onion encryption failed: {}", e))?;

            Ok(TaiorPacket {
                encrypted_payload: onion_encrypted,
                ikm: packet.ikm,
                ttl: packet.ttl,
                is_cover: packet.is_cover,
            })
        } else {
            // No circuit available — return packet with single-layer encryption
            Ok(packet)
        }
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
