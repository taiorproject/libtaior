use aorp::{DecisionConfig, DecisionEngine, EntropySource, MetricView, NeighborSet, PolicyConstraints};
use chacha20poly1305::{aead::Aead, aead::KeyInit, ChaCha20Poly1305, Key, Nonce};
use hkdf::Hkdf;
use rand_core::{OsRng, RngCore};
use sha2::Sha256;
use x25519_dalek::{EphemeralSecret, PublicKey};

fn main() {
    // 1) Handshake efímero X25519 y derivación de claves (HKDF-SHA256)
    let (initiator_sk, initiator_pk) = gen_keypair();
    let (responder_sk, responder_pk) = gen_keypair();

    let shared_initiator = initiator_sk.diffie_hellman(&responder_pk);
    let shared_responder = responder_sk.diffie_hellman(&initiator_pk);
    assert_eq!(shared_initiator.as_bytes(), shared_responder.as_bytes());

    let (aead_key, nonce) = kdf_to_aead(shared_initiator.as_bytes());
    let cipher = ChaCha20Poly1305::new(&aead_key);

    // 2) Empaquetado: payload fijo + padding + AEAD
    let payload = b"mensaje_demo";
    let padded = pad_payload(payload, 64);
    let ciphertext = cipher
        .encrypt(&nonce, padded.as_slice())
        .expect("encriptar");

    // 3) Simular hop decision con aorp-core
    let mut engine = DecisionEngine::new(DecisionConfig::new(Some(5)));
    let neighbors = NeighborSet::from_peers(["n1", "n2", "n3", "n4"]);
    let metrics = MetricView::builder()
        .add_latency_bucket(aorp::interfaces::types::NeighborId("n1".into()), aorp::interfaces::types::LatencyBucket::Low)
        .add_latency_bucket(aorp::interfaces::types::NeighborId("n2".into()), aorp::interfaces::types::LatencyBucket::Medium)
        .add_latency_bucket(aorp::interfaces::types::NeighborId("n3".into()), aorp::interfaces::types::LatencyBucket::High)
        .add_latency_bucket(aorp::interfaces::types::NeighborId("n4".into()), aorp::interfaces::types::LatencyBucket::Medium)
        .add_bandwidth_rank(aorp::interfaces::types::NeighborId("n1".into()), aorp::interfaces::types::BandwidthRank::High)
        .add_bandwidth_rank(aorp::interfaces::types::NeighborId("n2".into()), aorp::interfaces::types::BandwidthRank::Medium)
        .add_bandwidth_rank(aorp::interfaces::types::NeighborId("n3".into()), aorp::interfaces::types::BandwidthRank::Low)
        .add_bandwidth_rank(aorp::interfaces::types::NeighborId("n4".into()), aorp::interfaces::types::BandwidthRank::Medium)
        .build();

    let policies = PolicyConstraints::builder()
        .require_diversity(aorp::interfaces::types::DiversityLevel::Medium)
        .latency_weight(2)
        .bandwidth_weight(1)
        .avoid_loops(true)
        .build();

    let hop = engine.decide_next_hop(neighbors, metrics, EntropySource::secure_random(), policies);

    // 4) Decrypt y mostrar
    let decrypted = cipher
        .decrypt(&nonce, ciphertext.as_slice())
        .expect("decrypt");

    println!("Próximo salto: {:?}", hop);
    println!("Ciphertext (base16): {}", hex::encode(&ciphertext));
    println!("Payload desencriptado: {:?}", String::from_utf8_lossy(&decrypted));
}

fn gen_keypair() -> (EphemeralSecret, PublicKey) {
    let mut rng = OsRng;
    let sk = EphemeralSecret::new(&mut rng);
    let pk = PublicKey::from(&sk);
    (sk, pk)
}

fn kdf_to_aead(shared_secret: &[u8]) -> (Key, Nonce) {
    let hk = Hkdf::<Sha256>::new(None, shared_secret);
    let mut okm = [0u8; 44]; // 32 bytes key + 12 bytes nonce
    hk.expand(b"libtaior-demo", &mut okm).expect("hkdf expand");
    let key = Key::from_slice(&okm[..32]);
    let nonce = Nonce::from_slice(&okm[32..]);
    (key.clone(), *nonce)
}

fn pad_payload(payload: &[u8], target_len: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(target_len);
    out.extend_from_slice(payload);
    if payload.len() < target_len {
        let pad_len = target_len - payload.len();
        out.resize(payload.len() + pad_len, 0u8);
    }
    out
}
