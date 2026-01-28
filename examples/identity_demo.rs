use taior::identity::{EphemeralIdentity, TaiorAddress};

fn main() {
    println!("=== Taior Identity Demo ===\n");

    println!("Generando 3 identidades efímeras...\n");

    for i in 1..=3 {
        let identity = EphemeralIdentity::new();
        println!("Identidad {}:", i);
        println!("  Dirección: {}", identity.address.as_str());
        println!("  Formato: taior://<blake3-hash-256>");
        println!("  Amnesia: claves solo en RAM, no persistidas");
        println!();
    }

    println!("Características:");
    println!("✓ Cada instancia genera clave de sesión temporal");
    println!("✓ No existen identidades permanentes obligatorias");
    println!("✓ Direcciones derivadas criptográficamente (X25519 → BLAKE3)");
    println!("✓ Reiniciar = borrado criptográfico");
}
