use crate::packet::TaiorPacket;
use rand_core::{OsRng, RngCore};

pub struct CoverTrafficGenerator {
    enabled: bool,
    ratio: f32,
}

impl CoverTrafficGenerator {
    pub fn new(enabled: bool, ratio: f32) -> Self {
        Self { enabled, ratio }
    }

    pub fn should_send_cover(&self) -> bool {
        if !self.enabled {
            return false;
        }
        let mut rng = OsRng;
        let rand_val = (rng.next_u32() as f32) / (u32::MAX as f32);
        rand_val < self.ratio
    }

    pub fn generate_cover_packet(&self, padding_size: usize, ttl: u8) -> Result<TaiorPacket, String> {
        let mut dummy_payload = vec![0u8; 16];
        OsRng.fill_bytes(&mut dummy_payload);
        TaiorPacket::new(&dummy_payload, ttl, padding_size, true)
    }
}

impl Default for CoverTrafficGenerator {
    fn default() -> Self {
        Self::new(false, 0.3)
    }
}
