use taior::transport::RelayServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Taior Relay Server Demo ===\n");

    let bind_addr = "127.0.0.1:8888".parse()?;
    let relay = RelayServer::new(bind_addr).await?;

    println!("✓ Relay server escuchando en: {}", relay.local_addr());
    println!("✓ Autenticación efímera habilitada");
    println!("✓ Esperando clientes...\n");

    relay.run().await?;

    Ok(())
}
