use taior::{QuicTransport, QuicConfig, NatTraversal, RelayAuth, RelayClient, TaiorAddress, SendOptions, Taior};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    println!("=== Taior Transport Demo ===\n");

    // 1. NAT Traversal - Descubrir dirección pública
    println!("1. NAT Traversal - Descubriendo dirección pública...");
    let nat = NatTraversal::new("0.0.0.0:0".parse().unwrap());
    match nat.discover_public_addr().await {
        Ok(public_addr) => {
            println!("   ✓ Dirección pública descubierta: {}", public_addr);
        }
        Err(e) => {
            println!("   ✗ Error NAT traversal: {} (esperado en algunos entornos)", e);
        }
    }
    println!();

    // 2. Hole Punching - Preparar socket para peer-to-peer
    println!("2. Hole Punching - Preparando conexión P2P...");
    let _local_addr: SocketAddr = "0.0.0.0:0".parse().unwrap();
    let peer_addr: SocketAddr = "127.0.0.1:9999".parse().unwrap();
    
    match nat.hole_punch(peer_addr).await {
        Ok(_socket) => {
            println!("   ✓ Hole punching completado hacia {}", peer_addr);
            println!("   ✓ Socket preparado para comunicación directa");
        }
        Err(e) => {
            println!("   ✗ Error hole punching: {}", e);
        }
    }
    println!();

    // 3. QUIC Transport - Crear endpoint
    println!("3. QUIC Transport - Inicializando endpoint...");
    let config = QuicConfig {
        bind_addr: "127.0.0.1:0".parse().unwrap(),
        server_mode: false,
    };

    match QuicTransport::new(config).await {
        Ok(transport) => {
            println!("   ✓ QUIC endpoint creado");
            println!("   ✓ Dirección local: {}", transport.endpoint.local_addr().unwrap());
            println!("   ✓ TLS 1.3 con certificados efímeros");
        }
        Err(e) => {
            println!("   ✗ Error QUIC: {}", e);
        }
    }
    println!();

    // 4. Relay con autenticación efímera
    println!("4. Relay - Autenticación efímera...");
    let taior = Taior::new();
    let address = TaiorAddress(taior.address().to_string());
    let auth = RelayAuth::generate(&address);
    
    println!("   ✓ Token generado: {}...", &auth.token[..16]);
    println!("   ✓ Expira en: {} segundos", auth.expires_at - std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs());
    println!("   ✓ Válido: {}", auth.is_valid());
    println!();

    // 5. RelayClient - Envío via relay
    println!("5. RelayClient - Configurando cliente relay...");
    let relay_addr: SocketAddr = "127.0.0.1:8888".parse().unwrap();
    
    match RelayClient::new(relay_addr, auth).await {
        Ok(client) => {
            println!("   ✓ Cliente relay conectado a {}", relay_addr);
            println!("   ✓ Listo para enviar paquetes via relay");
            
            // Simular envío
            let mut taior = Taior::with_bootstrap(vec!["node1.taior.net".to_string()]);
            let packet = taior.send(b"Mensaje via relay", SendOptions::mix()).unwrap();
            let dest_addr = TaiorAddress("taior://destino".to_string());
            
            match client.send_via_relay(&packet, &dest_addr).await {
                Ok(_) => println!("   ✓ Paquete enviado via relay"),
                Err(e) => println!("   ✗ Error envío: {} (esperado sin servidor)", e),
            }
        }
        Err(e) => {
            println!("   ✗ Error relay client: {}", e);
        }
    }
    println!();

    println!("=== Demo completada ===");
    println!("\nCaracterísticas de transporte implementadas:");
    println!("✓ QUIC con TLS 1.3 y certificados efímeros");
    println!("✓ NAT traversal con STUN");
    println!("✓ Hole punching UDP para P2P");
    println!("✓ Relay con autenticación efímera (tokens temporales)");
    println!("✓ Soporte para conexiones directas y via relay");
}
