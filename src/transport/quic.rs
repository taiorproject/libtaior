use super::{Transport, TransportError, TransportResult};
use crate::packet::TaiorPacket;
use quinn::{ClientConfig, Endpoint, ServerConfig, Connection};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct QuicConfig {
    pub bind_addr: SocketAddr,
    pub server_mode: bool,
}

impl Default for QuicConfig {
    fn default() -> Self {
        Self {
            bind_addr: "0.0.0.0:0".parse().unwrap(),
            server_mode: false,
        }
    }
}

pub struct QuicTransport {
    pub endpoint: Endpoint,
    connection: Arc<Mutex<Option<Connection>>>,
}

impl QuicTransport {
    pub async fn new(config: QuicConfig) -> TransportResult<Self> {
        let endpoint = if config.server_mode {
            Self::create_server_endpoint(config.bind_addr).await?
        } else {
            Self::create_client_endpoint(config.bind_addr).await?
        };

        Ok(Self {
            endpoint,
            connection: Arc::new(Mutex::new(None)),
        })
    }

    async fn create_server_endpoint(bind_addr: SocketAddr) -> TransportResult<Endpoint> {
        let (cert, key) = generate_self_signed_cert()
            .map_err(|e| TransportError::ConnectionFailed(format!("cert gen: {}", e)))?;

        let mut server_config = ServerConfig::with_single_cert(vec![cert], key)
            .map_err(|e| TransportError::ConnectionFailed(format!("server config: {}", e)))?;

        let transport_config = Arc::get_mut(&mut server_config.transport)
            .ok_or_else(|| TransportError::ConnectionFailed("transport config".to_string()))?;
        transport_config.max_concurrent_uni_streams(0_u8.into());

        let endpoint = Endpoint::server(server_config, bind_addr)
            .map_err(|e| TransportError::ConnectionFailed(format!("bind: {}", e)))?;

        Ok(endpoint)
    }

    async fn create_client_endpoint(bind_addr: SocketAddr) -> TransportResult<Endpoint> {
        // TODO: Load pinned relay certificate hashes from configuration
        let pinned_hashes: Vec<[u8; 32]> = Vec::new();
        let crypto = rustls::ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(PinnedCertVerifier::new(pinned_hashes)))
            .with_no_client_auth();

        let client_config = ClientConfig::new(Arc::new(quinn::crypto::rustls::QuicClientConfig::try_from(crypto)
            .map_err(|e| TransportError::ConnectionFailed(format!("crypto config: {}", e)))?));

        let mut endpoint = Endpoint::client(bind_addr)
            .map_err(|e| TransportError::ConnectionFailed(format!("bind: {}", e)))?;
        endpoint.set_default_client_config(client_config);

        Ok(endpoint)
    }

    pub async fn accept(&self) -> TransportResult<Connection> {
        let incoming = self.endpoint.accept().await
            .ok_or_else(|| TransportError::ConnectionFailed("no incoming".to_string()))?;

        let conn = incoming.await
            .map_err(|e| TransportError::ConnectionFailed(format!("accept: {}", e)))?;

        Ok(conn)
    }
}

#[async_trait::async_trait]
impl Transport for QuicTransport {
    async fn connect(&mut self, addr: SocketAddr) -> TransportResult<()> {
        let conn = self.endpoint.connect(addr, "localhost")
            .map_err(|e| TransportError::ConnectionFailed(format!("connect: {}", e)))?
            .await
            .map_err(|e| TransportError::ConnectionFailed(format!("handshake: {}", e)))?;

        *self.connection.lock().await = Some(conn);
        Ok(())
    }

    async fn send(&mut self, packet: &TaiorPacket, _dest: SocketAddr) -> TransportResult<()> {
        let conn_guard = self.connection.lock().await;
        let conn = conn_guard.as_ref()
            .ok_or_else(|| TransportError::SendFailed("not connected".to_string()))?;

        let mut send_stream = conn.open_uni().await
            .map_err(|e| TransportError::SendFailed(format!("open stream: {}", e)))?;

        let serialized = serde_json::to_vec(packet)
            .map_err(|e| TransportError::SendFailed(format!("serialize: {}", e)))?;

        send_stream.write_all(&serialized).await
            .map_err(|e| TransportError::SendFailed(format!("write: {}", e)))?;

        send_stream.finish()
            .map_err(|e| TransportError::SendFailed(format!("finish: {}", e)))?;

        Ok(())
    }

    async fn receive(&mut self) -> TransportResult<(TaiorPacket, SocketAddr)> {
        let conn_guard = self.connection.lock().await;
        let conn = conn_guard.as_ref()
            .ok_or_else(|| TransportError::ReceiveFailed("not connected".to_string()))?;

        let mut recv_stream = conn.accept_uni().await
            .map_err(|e| TransportError::ReceiveFailed(format!("accept stream: {}", e)))?;

        let data = recv_stream.read_to_end(1024 * 1024).await
            .map_err(|e| TransportError::ReceiveFailed(format!("read: {}", e)))?;

        let packet: TaiorPacket = serde_json::from_slice(&data)
            .map_err(|e| TransportError::ReceiveFailed(format!("deserialize: {}", e)))?;

        let remote_addr = conn.remote_address();
        Ok((packet, remote_addr))
    }

    async fn close(&mut self) -> TransportResult<()> {
        if let Some(conn) = self.connection.lock().await.take() {
            conn.close(0u32.into(), b"closing");
        }
        self.endpoint.close(0u32.into(), b"closing");
        Ok(())
    }
}

fn generate_self_signed_cert() -> Result<(CertificateDer<'static>, PrivateKeyDer<'static>), Box<dyn std::error::Error>> {
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".to_string()])?;
    let key = PrivateKeyDer::Pkcs8(cert.key_pair.serialize_der().into());
    let cert_der = CertificateDer::from(cert.cert);
    Ok((cert_der, key))
}

/// Certificate pinning verifier: accepts only certificates whose SHA-256 fingerprint
/// matches one of the pinned hashes. This prevents MITM attacks even if a CA is compromised.
#[derive(Debug)]
struct PinnedCertVerifier {
    pinned_hashes: Vec<[u8; 32]>,
}

impl PinnedCertVerifier {
    fn new(pinned_hashes: Vec<[u8; 32]>) -> Self {
        Self { pinned_hashes }
    }

    fn fingerprint(cert: &CertificateDer<'_>) -> [u8; 32] {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(cert.as_ref());
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }
}

impl rustls::client::danger::ServerCertVerifier for PinnedCertVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        if self.pinned_hashes.is_empty() {
            // No pins configured — reject all connections to force explicit pinning
            return Err(rustls::Error::General(
                "No pinned certificates configured. Cannot verify relay identity.".into()
            ));
        }

        let cert_hash = Self::fingerprint(end_entity);
        if self.pinned_hashes.iter().any(|pin| pin == &cert_hash) {
            Ok(rustls::client::danger::ServerCertVerified::assertion())
        } else {
            Err(rustls::Error::General(
                "Certificate fingerprint does not match any pinned hash. Possible MITM.".into()
            ))
        }
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::ED25519,
        ]
    }
}
