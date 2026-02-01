use crate::packet::TaiorPacket;
use rand_core::{OsRng, RngCore};
use thiserror::Error;
#[cfg(target_arch = "wasm32")]
use js_sys;

#[derive(Error, Debug)]
pub enum CoverTrafficError {
    #[error("Error generando paquete cover: {0}")]
    GenerationError(String),
    #[error("Circuito no disponible")]
    NoCircuit,
}

#[derive(Debug, Clone)]
pub struct CoverTrafficConfig {
    pub enabled: bool,
    pub packets_per_second: f64,
    pub min_size: usize,
    pub max_size: usize,
    pub jitter_ms: u64,
}

impl Default for CoverTrafficConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            packets_per_second: 2.0,
            min_size: 512,
            max_size: 2048,
            jitter_ms: 500,
        }
    }
}

pub struct CoverTrafficGenerator {
    config: CoverTrafficConfig,
    last_sent: u64,
}

impl CoverTrafficGenerator {
    pub fn new(config: CoverTrafficConfig) -> Self {
        Self {
            config,
            last_sent: current_timestamp_ms(),
        }
    }

    pub fn should_send_cover(&mut self) -> bool {
        if !self.config.enabled || self.config.packets_per_second <= 0.0 {
            return false;
        }

        let now = current_timestamp_ms();
        let interval_ms = (1000.0 / self.config.packets_per_second) as u64;
        let jitter = (OsRng.next_u64() % (self.config.jitter_ms * 2)) as i64 - self.config.jitter_ms as i64;
        let target_interval = (interval_ms as i64 + jitter).max(0) as u64;

        if now - self.last_sent >= target_interval {
            self.last_sent = now;
            true
        } else {
            false
        }
    }

    pub fn generate_cover_packet(&self) -> Result<Vec<u8>, CoverTrafficError> {
        let size = if self.config.min_size == self.config.max_size {
            self.config.min_size
        } else {
            let range = self.config.max_size - self.config.min_size;
            self.config.min_size + (OsRng.next_u64() as usize % range)
        };

        let mut payload = vec![0u8; size];
        OsRng.fill_bytes(&mut payload);

        Ok(payload)
    }

    pub fn wrap_in_packet(&self, payload: Vec<u8>) -> Result<TaiorPacket, CoverTrafficError> {
        TaiorPacket::new(&payload, 3, 0, true)
            .map_err(|e| CoverTrafficError::GenerationError(e))
    }
}

pub struct AdaptiveCoverTraffic {
    generator: CoverTrafficGenerator,
    real_traffic_count: u64,
    cover_traffic_count: u64,
    target_ratio: f64,
    window_start: u64,
    window_duration_ms: u64,
}

impl AdaptiveCoverTraffic {
    pub fn new(config: CoverTrafficConfig, target_ratio: f64) -> Self {
        Self {
            generator: CoverTrafficGenerator::new(config),
            real_traffic_count: 0,
            cover_traffic_count: 0,
            target_ratio,
            window_start: current_timestamp_ms(),
            window_duration_ms: 60000,
        }
    }

    pub fn on_real_traffic(&mut self) {
        self.real_traffic_count += 1;
        self.check_window_reset();
    }

    pub fn should_send_adaptive_cover(&mut self) -> bool {
        self.check_window_reset();

        if self.real_traffic_count == 0 {
            return self.generator.should_send_cover();
        }

        let current_ratio = self.cover_traffic_count as f64 / self.real_traffic_count as f64;
        
        if current_ratio < self.target_ratio {
            let deficit = (self.target_ratio * self.real_traffic_count as f64) - self.cover_traffic_count as f64;
            let probability = (deficit / 10.0).min(1.0);
            
            if (OsRng.next_u64() as f64 / u64::MAX as f64) < probability {
                self.cover_traffic_count += 1;
                return true;
            }
        }

        false
    }

    fn check_window_reset(&mut self) {
        let now = current_timestamp_ms();
        if now - self.window_start >= self.window_duration_ms {
            self.real_traffic_count = 0;
            self.cover_traffic_count = 0;
            self.window_start = now;
        }
    }

    pub fn generate_cover_packet(&self) -> Result<Vec<u8>, CoverTrafficError> {
        self.generator.generate_cover_packet()
    }

    pub fn wrap_in_packet(&self, payload: Vec<u8>) -> Result<TaiorPacket, CoverTrafficError> {
        self.generator.wrap_in_packet(payload)
    }

    pub fn stats(&self) -> (u64, u64, f64) {
        let ratio = if self.real_traffic_count > 0 {
            self.cover_traffic_count as f64 / self.real_traffic_count as f64
        } else {
            0.0
        };
        (self.real_traffic_count, self.cover_traffic_count, ratio)
    }
}

fn current_timestamp_ms() -> u64 {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }
    
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::Date::now() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cover_traffic_generation() {
        let config = CoverTrafficConfig {
            enabled: true,
            packets_per_second: 10.0,
            min_size: 512,
            max_size: 1024,
            jitter_ms: 100,
        };

        let generator = CoverTrafficGenerator::new(config);
        let packet = generator.generate_cover_packet().unwrap();
        
        assert!(packet.len() >= 512);
        assert!(packet.len() <= 1024);
    }

    #[test]
    fn test_adaptive_cover_traffic() {
        let config = CoverTrafficConfig::default();
        let mut adaptive = AdaptiveCoverTraffic::new(config, 0.5);

        adaptive.on_real_traffic();
        adaptive.on_real_traffic();

        let (real, cover, ratio) = adaptive.stats();
        assert_eq!(real, 2);
    }
}
