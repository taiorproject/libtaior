use taior::transport::{QuicTransport, QuicConfig, Transport};
use taior::{Taior, SendOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Taior QUIC Client Demo ===\n");

    let config = QuicConfig {
        bind_addr: "0.0.0.0:0".parse()?,
        server_mode: false,
    };

    let mut transport = QuicTransport::new(config).await?;
    println!("✓ Cliente QUIC creado");

    let server_addr = "127.0.0.1:5000".parse()?;
    println!("✓ Conectando a servidor: {}...", server_addr);

    transport.connect(server_addr).await?;
    println!("✓ Conexión QUIC establecida\n");

    let mut taior = Taior::new();
    println!("Identidad: {}\n", taior.address());

    for i in 1..=3 {
        let message = format!("Mensaje {} via QUIC", i);
        let packet = taior.send(message.as_bytes(), SendOptions::fast())?;
        
        println!("Enviando mensaje {}...", i);
        transport.send(&packet, server_addr).await?;
        println!("  ✓ Enviado: {} bytes cifrados\n", packet.size());
        
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    println!("✓ Demo completada");
    transport.close().await?;

    Ok(())
}
