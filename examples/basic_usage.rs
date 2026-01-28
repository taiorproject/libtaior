use taior::{Taior, SendOptions, RoutingMode};

fn main() {
    println!("=== Taior Basic Usage Demo ===\n");

    let mut taior = Taior::with_bootstrap(vec![
        "node1.taior.net".to_string(),
        "node2.taior.net".to_string(),
        "node3.taior.net".to_string(),
    ]);

    println!("Identidad efímera generada: {}", taior.address());
    println!();

    println!("1. Modo Fast (baja latencia, 1-2 saltos):");
    let message = b"Hola desde Taior en modo Fast";
    match taior.send(message, SendOptions::fast()) {
        Ok(packet) => {
            println!("   ✓ Paquete creado: {} bytes cifrados", packet.size());
            println!("   ✓ TTL: {}", packet.ttl);
            println!("   ✓ Cover traffic: {}", packet.is_cover);
        }
        Err(e) => println!("   ✗ Error: {}", e),
    }
    println!();

    println!("2. Modo Mix (alta privacidad, 3-5 saltos, cover traffic):");
    taior.enable_cover_traffic(true, 0.3);
    let message = b"Mensaje privado en modo Mix";
    match taior.send(message, SendOptions::mix()) {
        Ok(packet) => {
            println!("   ✓ Paquete creado: {} bytes cifrados", packet.size());
            println!("   ✓ TTL: {}", packet.ttl);
            println!("   ✓ Padding indistinguible aplicado");
        }
        Err(e) => println!("   ✗ Error: {}", e),
    }
    println!();

    println!("3. Modo Adaptive (balance dinámico):");
    let message = b"Mensaje adaptativo";
    match taior.send(message, SendOptions::adaptive()) {
        Ok(packet) => {
            println!("   ✓ Paquete creado: {} bytes cifrados", packet.size());
            println!("   ✓ TTL: {}", packet.ttl);
        }
        Err(e) => println!("   ✗ Error: {}", e),
    }
    println!();

    println!("4. Modo Custom (configuración manual):");
    let message = b"Mensaje custom";
    match taior.send(message, SendOptions::custom(RoutingMode::Mix, 3)) {
        Ok(packet) => {
            println!("   ✓ Paquete creado: {} bytes cifrados", packet.size());
            println!("   ✓ TTL: {} (personalizado)", packet.ttl);
        }
        Err(e) => println!("   ✗ Error: {}", e),
    }
    println!();

    println!("=== Demo completada ===");
    println!("\nCaracterísticas implementadas:");
    println!("✓ Identidad efímera (taior://<hash>)");
    println!("✓ Modos Fast/Mix/Adaptive");
    println!("✓ AORP (enrutamiento probabilístico)");
    println!("✓ Cifrado AEAD (ChaCha20-Poly1305)");
    println!("✓ Padding indistinguible");
    println!("✓ Cover traffic configurable");
    println!("✓ Descubrimiento de nodos");
    println!("✓ API simple: Taior.send(data, options)");
}
