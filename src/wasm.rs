use wasm_bindgen::prelude::*;
use crate::{Taior, SendOptions, RoutingMode, CircuitBuilder, TaiorAddress};

#[wasm_bindgen]
pub struct TaiorWasm {
    inner: Taior,
    circuit_builder: CircuitBuilder,
}

#[wasm_bindgen]
impl TaiorWasm {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { 
            inner: Taior::new(),
            circuit_builder: CircuitBuilder::new(3, 5, 600),
        }
    }

    #[wasm_bindgen(js_name = withBootstrap)]
    pub fn with_bootstrap(bootstrap: Vec<String>) -> Self {
        Self {
            inner: Taior::with_bootstrap(bootstrap),
            circuit_builder: CircuitBuilder::new(3, 5, 600),
        }
    }

    pub fn address(&self) -> String {
        self.inner.address().to_string()
    }

    #[wasm_bindgen(js_name = addNode)]
    pub fn add_node(&mut self, node: String) {
        self.inner.add_node(node.clone());
        let address = TaiorAddress::generate();
        self.circuit_builder.add_node(node, address);
    }

    #[wasm_bindgen(js_name = removeNode)]
    pub fn remove_node(&mut self, node: String) {
        self.circuit_builder.remove_node(&node);
    }

    #[wasm_bindgen(js_name = decideNextHop)]
    pub fn decide_next_hop(&mut self, candidates: Vec<String>, remaining_hops: usize) -> Option<String> {
        if candidates.is_empty() {
            return None;
        }
        
        let mode_config = crate::modes::ModeConfig {
            hops: remaining_hops as u8,
            cover_traffic_rate: 0.0,
            padding_size: 0,
        };
        
        let mut router = crate::routing::Router::new();
        router.decide_next_hop(candidates, &mode_config)
    }

    #[wasm_bindgen(js_name = buildCircuit)]
    pub fn build_circuit(&mut self, hops: usize) -> Result<Vec<u8>, JsValue> {
        let circuit = self.circuit_builder.build_circuit(hops)
            .map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
        
        Ok(circuit.id.to_vec())
    }

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

        Ok(packet.encrypted_payload.clone())
    }

    #[wasm_bindgen(js_name = enableCoverTraffic)]
    pub fn enable_cover_traffic(&mut self, enabled: bool, ratio: f32) {
        self.inner.enable_cover_traffic(enabled, ratio);
    }
}

#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}
