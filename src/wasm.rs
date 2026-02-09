use wasm_bindgen::prelude::*;
use crate::{Taior, SendOptions, RoutingMode};

#[wasm_bindgen]
pub struct TaiorWasm {
    inner: Taior,
}

#[wasm_bindgen]
impl TaiorWasm {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { 
            inner: Taior::new(),
        }
    }

    #[wasm_bindgen(js_name = withBootstrap)]
    pub fn with_bootstrap(bootstrap: Vec<String>) -> Self {
        Self {
            inner: Taior::with_bootstrap(bootstrap),
        }
    }

    pub fn address(&self) -> String {
        self.inner.address().to_string()
    }

    #[wasm_bindgen(js_name = addNode)]
    pub fn add_node(&mut self, node: String) {
        self.inner.add_node(node);
    }

    #[wasm_bindgen(js_name = decideNextHop)]
    pub fn decide_next_hop(&mut self, candidates: Vec<String>, remaining_hops: usize) -> Option<String> {
        if candidates.is_empty() {
            return None;
        }
        
        let mode_config = crate::modes::ModeConfig {
            mode: crate::modes::RoutingMode::Adaptive,
            hops: remaining_hops as u8,
            cover_traffic: false,
            jitter_ms: None,
            padding_size: 0,
        };
        
        let mut router = crate::routing::Router::new();
        router.decide_next_hop(candidates, &mode_config)
    }

    /// Returns the full serialized packet (encrypted_payload + ikm) as a single byte array.
    /// Format: [4 bytes: payload_len (big-endian)] [encrypted_payload] [ikm]
    pub fn send(&mut self, data: &[u8], mode: String) -> Result<Vec<u8>, JsValue> {
        let routing_mode = match mode.as_str() {
            "fast" => RoutingMode::Fast,
            "mix" => RoutingMode::Mix,
            "adaptive" => RoutingMode::Adaptive,
            _ => RoutingMode::Adaptive,
        };

        let hops = match mode.as_str() {
            "fast" => 1,
            "mix" => 4,
            _ => 2,
        };

        let opts = SendOptions::custom(routing_mode, hops);
        let packet = self.inner.send(data, opts)
            .map_err(|e| JsValue::from_str(&e))?;

        // Serialize: [4 bytes payload_len] [encrypted_payload] [ikm]
        let payload_len = packet.encrypted_payload.len() as u32;
        let mut result = Vec::with_capacity(4 + packet.encrypted_payload.len() + packet.ikm.len());
        result.extend_from_slice(&payload_len.to_be_bytes());
        result.extend_from_slice(&packet.encrypted_payload);
        result.extend_from_slice(&packet.ikm);

        Ok(result)
    }

    #[wasm_bindgen(js_name = enableCoverTraffic)]
    pub fn enable_cover_traffic(&mut self, enabled: bool, ratio: f32) {
        self.inner.enable_cover_traffic(enabled, ratio);
    }
}

pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}
