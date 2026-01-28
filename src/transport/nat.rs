use super::{TransportError, TransportResult};
use std::net::{SocketAddr, IpAddr};
use tokio::net::UdpSocket;
use std::time::Duration;

pub struct NatTraversal {
    local_addr: SocketAddr,
    stun_servers: Vec<String>,
}

impl NatTraversal {
    pub fn new(local_addr: SocketAddr) -> Self {
        Self {
            local_addr,
            stun_servers: vec![
                "stun.l.google.com:19302".to_string(),
                "stun1.l.google.com:19302".to_string(),
            ],
        }
    }

    pub fn with_stun_servers(mut self, servers: Vec<String>) -> Self {
        self.stun_servers = servers;
        self
    }

    pub async fn discover_public_addr(&self) -> TransportResult<SocketAddr> {
        for stun_server in &self.stun_servers {
            match self.query_stun(stun_server).await {
                Ok(addr) => return Ok(addr),
                Err(_) => continue,
            }
        }
        Err(TransportError::NatTraversalFailed("all STUN servers failed".to_string()))
    }

    async fn query_stun(&self, stun_server: &str) -> TransportResult<SocketAddr> {
        let socket = UdpSocket::bind(self.local_addr).await
            .map_err(|e| TransportError::NatTraversalFailed(format!("bind: {}", e)))?;

        let stun_addr: SocketAddr = tokio::net::lookup_host(stun_server).await
            .map_err(|e| TransportError::NatTraversalFailed(format!("resolve: {}", e)))?
            .next()
            .ok_or_else(|| TransportError::NatTraversalFailed("no address".to_string()))?;

        let request = build_stun_binding_request();
        socket.send_to(&request, stun_addr).await
            .map_err(|e| TransportError::NatTraversalFailed(format!("send: {}", e)))?;

        let mut buf = vec![0u8; 1024];
        let timeout = tokio::time::timeout(Duration::from_secs(5), socket.recv_from(&mut buf)).await
            .map_err(|_| TransportError::NatTraversalFailed("timeout".to_string()))?
            .map_err(|e| TransportError::NatTraversalFailed(format!("recv: {}", e)))?;

        let (len, _) = timeout;
        parse_stun_response(&buf[..len])
    }

    pub async fn hole_punch(&self, peer_addr: SocketAddr) -> TransportResult<UdpSocket> {
        let socket = UdpSocket::bind(self.local_addr).await
            .map_err(|e| TransportError::NatTraversalFailed(format!("bind: {}", e)))?;

        for _ in 0..5 {
            socket.send_to(b"PUNCH", peer_addr).await
                .map_err(|e| TransportError::NatTraversalFailed(format!("punch: {}", e)))?;
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        Ok(socket)
    }
}

fn build_stun_binding_request() -> Vec<u8> {
    let mut request = vec![0u8; 20];
    request[0] = 0x00;
    request[1] = 0x01;
    request[2] = 0x00;
    request[3] = 0x00;
    request[4..8].copy_from_slice(b"\x21\x12\xa4\x42");
    for i in 8..20 {
        request[i] = rand::random();
    }
    request
}

fn parse_stun_response(data: &[u8]) -> TransportResult<SocketAddr> {
    if data.len() < 20 {
        return Err(TransportError::NatTraversalFailed("response too short".to_string()));
    }

    let msg_type = u16::from_be_bytes([data[0], data[1]]);
    if msg_type != 0x0101 {
        return Err(TransportError::NatTraversalFailed("not binding response".to_string()));
    }

    let mut pos = 20;
    while pos + 4 <= data.len() {
        let attr_type = u16::from_be_bytes([data[pos], data[pos + 1]]);
        let attr_len = u16::from_be_bytes([data[pos + 2], data[pos + 3]]) as usize;
        pos += 4;

        if pos + attr_len > data.len() {
            break;
        }

        if attr_type == 0x0020 {
            if attr_len >= 8 {
                let family = data[pos + 1];
                let port = u16::from_be_bytes([data[pos + 2], data[pos + 3]]) ^ 0x2112;
                
                let ip = if family == 0x01 {
                    let octets = [
                        data[pos + 4] ^ 0x21,
                        data[pos + 5] ^ 0x12,
                        data[pos + 6] ^ 0xa4,
                        data[pos + 7] ^ 0x42,
                    ];
                    IpAddr::from(octets)
                } else {
                    return Err(TransportError::NatTraversalFailed("IPv6 not supported".to_string()));
                };

                return Ok(SocketAddr::new(ip, port));
            }
        }

        pos += attr_len;
        pos = (pos + 3) & !3;
    }

    Err(TransportError::NatTraversalFailed("no XOR-MAPPED-ADDRESS".to_string()))
}
