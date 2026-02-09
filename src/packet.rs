use chacha20poly1305::{aead::Aead, aead::KeyInit, ChaCha20Poly1305, Key, Nonce};
use hkdf::Hkdf;
use rand_core::{OsRng, RngCore};
use sha2::Sha256;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaiorPacket {
    pub encrypted_payload: Vec<u8>,
    /// Ephemeral key material needed by the receiver to derive decryption key
    pub ikm: Vec<u8>,
    pub ttl: u8,
    pub is_cover: bool,
}

impl TaiorPacket {
    pub fn new(payload: &[u8], ttl: u8, padding_size: usize, is_cover: bool) -> Result<Self, String> {
        let target = if padding_size > 0 { padding_size } else { payload.len() };
        let padded = pad_payload(payload, target);
        let (key, nonce, ikm) = derive_packet_key();
        let cipher = ChaCha20Poly1305::new(&key);
        
        let encrypted_payload = cipher
            .encrypt(&nonce, padded.as_slice())
            .map_err(|e| format!("encrypt error: {:?}", e))?;

        Ok(Self {
            encrypted_payload,
            ikm: ikm.to_vec(),
            ttl,
            is_cover,
        })
    }

    pub fn decrypt_with_ikm(&self) -> Result<Vec<u8>, String> {
        let (key, nonce) = derive_key_from_ikm(&self.ikm)?;
        let cipher = ChaCha20Poly1305::new(&key);
        cipher
            .decrypt(&nonce, self.encrypted_payload.as_slice())
            .map_err(|e| format!("decrypt error: {:?}", e))
    }

    pub fn decrypt(&self, key: &Key, nonce: &Nonce) -> Result<Vec<u8>, String> {
        let cipher = ChaCha20Poly1305::new(key);
        cipher
            .decrypt(nonce, self.encrypted_payload.as_slice())
            .map_err(|e| format!("decrypt error: {:?}", e))
    }

    pub fn size(&self) -> usize {
        self.encrypted_payload.len() + self.ikm.len()
    }
}

pub fn pad_payload(payload: &[u8], target_len: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(target_len);
    out.extend_from_slice(payload);
    if payload.len() < target_len {
        let pad_len = target_len - payload.len();
        let mut padding = vec![0u8; pad_len];
        OsRng.fill_bytes(&mut padding);
        out.extend_from_slice(&padding);
    }
    out.truncate(target_len);
    out
}

pub fn derive_packet_key() -> (Key, Nonce, [u8; 32]) {
    let mut ikm = [0u8; 32];
    OsRng.fill_bytes(&mut ikm);
    
    let (key, nonce) = derive_key_from_ikm(&ikm).expect("hkdf expand from fresh ikm");
    (key, nonce, ikm)
}

pub fn derive_key_from_ikm(ikm: &[u8]) -> Result<(Key, Nonce), String> {
    if ikm.len() < 32 {
        return Err("IKM too short: expected at least 32 bytes".to_string());
    }
    let hk = Hkdf::<Sha256>::new(None, ikm);
    let mut okm = [0u8; 44];
    hk.expand(b"taior-packet-v1", &mut okm)
        .map_err(|e| format!("hkdf expand error: {:?}", e))?;
    
    let key = *Key::from_slice(&okm[..32]);
    let nonce = *Nonce::from_slice(&okm[32..]);
    Ok((key, nonce))
}
