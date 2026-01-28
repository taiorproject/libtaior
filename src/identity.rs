use blake3::Hasher;
use rand_core::OsRng;
use x25519_dalek::{EphemeralSecret, PublicKey};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaiorAddress(pub String);

impl TaiorAddress {
    pub fn generate() -> (EphemeralSecret, Self) {
        let sk = EphemeralSecret::random_from_rng(OsRng);
        let pk = PublicKey::from(&sk);
        let addr = Self::from_public_key(&pk);
        (sk, addr)
    }

    pub fn from_public_key(pk: &PublicKey) -> Self {
        let mut hasher = Hasher::new();
        hasher.update(pk.as_bytes());
        let hash = hasher.finalize();
        let addr = format!("taior://{}", hex::encode(&hash.as_bytes()[..32]));
        TaiorAddress(addr)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

pub struct EphemeralIdentity {
    pub secret: EphemeralSecret,
    pub address: TaiorAddress,
}

impl EphemeralIdentity {
    pub fn new() -> Self {
        let (secret, address) = TaiorAddress::generate();
        Self { secret, address }
    }
}

impl Default for EphemeralIdentity {
    fn default() -> Self {
        Self::new()
    }
}
