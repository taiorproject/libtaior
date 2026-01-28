use taior::transport::{QuicTransport, QuicConfig, Transport};
use taior::{Taior, SendOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Taior QUIC Server Demo ===\n");

    let config = QuicConfig {
        bind_addr: "127.0.0.1:5000".parse()?,
        server_mode: true,
    };

    let transport = QuicTransport::new(config).await?;
    println!("✓ Servidor QUIC escuchando en: {}", transport.endpoint.local_addr()?);
    println!("✓ Esperando conexiones...\n");

    loop {
        match transport.accept().await {
            Ok(conn) => {
                println!("✓ Nueva conexión desde: {}", conn.remote_address());
                
                tokio::spawn(async move {
                    let mut transport_conn = QuicTransport {
                        endpoint: conn.clone().into(),
                        connection: std::sync::Arc::new(tokio::sync::Mutex::new(Some(conn))),
                    };

                    loop {
                        match transport_conn.receive().await {
                            Ok((packet, addr)) => {
                                println!("  → Paquete recibido de {}", addr);
                                println!("    TTL: {}, Tamaño: {} bytes, Cover: {}", 
                                    packet.ttl, packet.size(), packet.is_cover);
                            }
                            Err(e) => {
                                println!("  ✗ Error recibiendo: {}", e);
                                break;
                            }
                        }
                    }
                });
            }
            Err(e) => {
                println!("✗ Error aceptando conexión: {}", e);
            }
        }
    }
}
