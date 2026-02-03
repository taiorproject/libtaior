use crate::identity::TaiorAddress;
use crate::routing::Router;
use chacha20poly1305::{aead::Aead, aead::KeyInit, ChaCha20Poly1305, Key, Nonce};
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
use js_sys;

#[derive(Error, Debug)]
pub enum CircuitError {
    #[error("No hay suficientes nodos para construir circuito de {0} hops")]
    InsufficientNodes(usize),
    #[error("Error de cifrado: {0}")]
    EncryptionError(String),
    #[error("Circuito expirado")]
    CircuitExpired,
    #[error("Hop {0} no responde")]
    HopTimeout(usize),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitNode {
    pub address: TaiorAddress,
    pub shared_key: Vec<u8>,
    pub nonce: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Circuit {
    pub id: [u8; 16],
    pub nodes: Vec<CircuitNode>,
    pub created_at: u64,
    pub ttl_seconds: u64,
}

impl Circuit {
    pub fn new(nodes: Vec<CircuitNode>, ttl_seconds: u64) -> Self {
        let mut id = [0u8; 16];
        OsRng.fill_bytes(&mut id);
        
        Self {
            id,
            nodes,
            created_at: current_timestamp(),
            ttl_seconds,
        }
    }

    pub fn is_expired(&self) -> bool {
        current_timestamp() - self.created_at > self.ttl_seconds
    }

    pub fn hop_count(&self) -> usize {
        self.nodes.len()
    }
}

pub struct CircuitBuilder {
    router: Router,
    available_nodes: HashMap<String, TaiorAddress>,
    min_hops: usize,
    max_hops: usize,
    circuit_ttl: u64,
}

impl CircuitBuilder {
    pub fn new(min_hops: usize, max_hops: usize, circuit_ttl: u64) -> Self {
        Self {
            router: Router::new(),
            available_nodes: HashMap::new(),
            min_hops,
            max_hops,
            circuit_ttl,
        }
    }

    pub fn add_node(&mut self, id: String, address: TaiorAddress) {
        self.available_nodes.insert(id, address);
    }

    pub fn remove_node(&mut self, id: &str) {
        self.available_nodes.remove(id);
    }

    pub fn build_circuit(&mut self, target_hops: usize) -> Result<Circuit, CircuitError> {
        if target_hops < self.min_hops {
            return Err(CircuitError::InsufficientNodes(self.min_hops));
        }

        let hops = target_hops.min(self.max_hops);
        
        if self.available_nodes.len() < hops {
            return Err(CircuitError::InsufficientNodes(hops));
        }

        let mut circuit_nodes = Vec::with_capacity(hops);
        let mut used_nodes: Vec<String> = Vec::new();

        for _ in 0..hops {
            let available: Vec<String> = self.available_nodes
                .keys()
                .filter(|k| !used_nodes.contains(k))
                .cloned()
                .collect();

            if available.is_empty() {
                return Err(CircuitError::InsufficientNodes(hops));
            }

            let next_hop = if available.len() == 1 {
                available[0].clone()
            } else {
                let mode_config = crate::modes::ModeConfig {
                    mode: crate::modes::RoutingMode::Adaptive,
                    hops: (hops - circuit_nodes.len()) as u8,
                    cover_traffic: false,
                    jitter_ms: None,
                    padding_size: 0,
                };

                self.router
                    .decide_next_hop(available.clone(), &mode_config)
                    .ok_or_else(|| CircuitError::InsufficientNodes(hops))?
            };

            let address = self.available_nodes.get(&next_hop)
                .ok_or_else(|| CircuitError::InsufficientNodes(hops))?
                .clone();

            let (shared_key, nonce) = generate_hop_keys();

            circuit_nodes.push(CircuitNode {
                address,
                shared_key,
                nonce,
            });

            used_nodes.push(next_hop);
        }

        Ok(Circuit::new(circuit_nodes, self.circuit_ttl))
    }
}

pub struct OnionEncryptor {
    circuit: Circuit,
}

impl OnionEncryptor {
    pub fn new(circuit: Circuit) -> Self {
        Self { circuit }
    }

    pub fn encrypt_onion(&self, payload: &[u8]) -> Result<Vec<u8>, CircuitError> {
        if self.circuit.is_expired() {
            return Err(CircuitError::CircuitExpired);
        }

        let mut encrypted = payload.to_vec();

        for node in self.circuit.nodes.iter().rev() {
            encrypted = self.encrypt_layer(&encrypted, node)?;
        }

        Ok(encrypted)
    }

    fn encrypt_layer(&self, data: &[u8], node: &CircuitNode) -> Result<Vec<u8>, CircuitError> {
        let key = Key::from_slice(&node.shared_key);
        let nonce = Nonce::from_slice(&node.nonce);
        let cipher = ChaCha20Poly1305::new(key);

        cipher
            .encrypt(nonce, data)
            .map_err(|e| CircuitError::EncryptionError(format!("{:?}", e)))
    }

    pub fn decrypt_layer(&self, data: &[u8], hop_index: usize) -> Result<Vec<u8>, CircuitError> {
        if hop_index >= self.circuit.nodes.len() {
            return Err(CircuitError::EncryptionError("Hop index fuera de rango".into()));
        }

        let node = &self.circuit.nodes[hop_index];
        let key = Key::from_slice(&node.shared_key);
        let nonce = Nonce::from_slice(&node.nonce);
        let cipher = ChaCha20Poly1305::new(key);

        cipher
            .decrypt(nonce, data)
            .map_err(|e| CircuitError::EncryptionError(format!("{:?}", e)))
    }
}

fn generate_hop_keys() -> (Vec<u8>, Vec<u8>) {
    let mut key = vec![0u8; 32];
    let mut nonce = vec![0u8; 12];
    OsRng.fill_bytes(&mut key);
    OsRng.fill_bytes(&mut nonce);
    (key, nonce)
}

fn current_timestamp() -> u64 {
    #[cfg(not(target_arch = "wasm32"))]
    {
        return std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }
    
    #[cfg(all(target_arch = "wasm32", feature = "wasm"))]
    {
        return (js_sys::Date::now() / 1000.0) as u64;
    }
    
    #[cfg(all(target_arch = "wasm32", not(feature = "wasm")))]
    {
        0 // Fallback para WASM sin feature
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_creation() {
        let nodes = vec![
            CircuitNode {
                address: TaiorAddress::generate().1,
                shared_key: vec![0u8; 32],
                nonce: vec![0u8; 12],
            },
            CircuitNode {
                address: TaiorAddress::generate().1,
                shared_key: vec![1u8; 32],
                nonce: vec![1u8; 12],
            },
        ];

        let circuit = Circuit::new(nodes, 3600);
        assert_eq!(circuit.hop_count(), 2);
        assert!(!circuit.is_expired());
    }

    #[test]
    fn test_onion_encryption() {
        let nodes = vec![
            CircuitNode {
                address: TaiorAddress::generate().1,
                shared_key: vec![0u8; 32],
                nonce: vec![0u8; 12],
            },
        ];

        let circuit = Circuit::new(nodes, 3600);
        let encryptor = OnionEncryptor::new(circuit);
        
        let payload = b"test message";
        let encrypted = encryptor.encrypt_onion(payload).unwrap();
        
        assert_ne!(encrypted, payload);
        assert!(encrypted.len() > payload.len());
    }
}
