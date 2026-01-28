# Taior: The Amnesic Incognito Oblivious Routing

**Estado: investigación experimental. No es un producto de protección de privacidad listo para producción.**

Taior es una librería de enrutamiento anónimo embebible diseñada para ocultar metadatos de red (IP origen, IP destino, patrones de tráfico) sin exigir al usuario instalar redes externas como Tor, I2P o Lokinet. Se integra directamente en aplicaciones (chat, wallets, APIs, P2P, web, mobile).

**Taior no es una VPN, no es una red monolítica, no es un reemplazo de Tor. Es un componente de red embebible que permite a cualquier software hablar de forma privada por defecto.**

> Buscamos crítica activa de investigadores y revisiones de seguridad. Por favor abra issues o envíe comentarios formales.

## Características principales

- **Identidad efímera**: Direcciones `taior://<hash>` sin persistencia, amnesia por diseño
- **Modos configurables**: Fast (1-2 saltos), Mix (3-5 saltos + cover traffic), Adaptive
- **AORP**: Enrutamiento probabilístico hop-by-hop sin rutas precomputadas
- **Cifrado multicapa**: ChaCha20-Poly1305 AEAD + X25519 + HKDF-SHA256
- **Paquetes indistinguibles**: Padding fijo, cover traffic configurable
- **Transporte QUIC**: TLS 1.3 con certificados efímeros, conexiones seguras
- **NAT traversal**: STUN para descubrimiento de IP pública, hole punching UDP
- **Relays/Bridges**: Autenticación efímera con tokens temporales, sin identidades persistentes
- **API simple**: `Taior.send(data, options)` — integrable como librería
- **Stateless**: Sin estado persistente, claves solo en RAM
- **Descubrimiento de nodos**: Listas embebidas, DHT volátil (preparado)

## Estado y advertencia

- **No apto para producción.** Sin auditoría, sin garantías de anonimato robusto.
- No usar en entornos que requieran privacidad fuerte o protección contra adversarios globales.
- Diseñado para experimentos controlados y revisión crítica.

## Inicio rápido

### Instalación

```toml
[dependencies]
taior = { git = "https://github.com/taiorproject/libtaior", branch = "main" }
```

### Uso básico

```rust
use taior::{Taior, SendOptions};

fn main() {
    // 1. Crear instancia con identidad efímera
    let mut taior = Taior::with_bootstrap(vec![
        "node1.taior.net".to_string(),
        "node2.taior.net".to_string(),
    ]);

    println!("Mi dirección: {}", taior.address());

    // 2. Enviar mensaje en modo Fast (baja latencia)
    let packet = taior.send(b"Hola Taior", SendOptions::fast()).unwrap();
    println!("Paquete cifrado: {} bytes", packet.size());

    // 3. Enviar en modo Mix (alta privacidad)
    taior.enable_cover_traffic(true, 0.3);
    let packet = taior.send(b"Mensaje privado", SendOptions::mix()).unwrap();

    // 4. Modo custom
    let packet = taior.send(
        b"Custom",
        SendOptions::custom(taior::RoutingMode::Mix, 3)
    ).unwrap();
}
```

### Ejemplos

```bash
# Uso básico de API
cargo run --example basic_usage
cargo run --example identity_demo

# Transporte y red
cargo run --example transport_demo

# Cliente/servidor QUIC (ejecutar en terminales separadas)
cargo run --example quic_server
cargo run --example quic_client

# Relay server
cargo run --example relay_server
```

## Estructura del repositorio

- `src/`: Código fuente de la librería Taior
  - `api.rs`: API pública (`Taior`, `SendOptions`)
  - `identity.rs`: Identidades efímeras y direcciones `taior://`
  - `modes.rs`: Modos Fast/Mix/Adaptive
  - `packet.rs`: Empaquetado, cifrado AEAD, padding
  - `routing.rs`: Integración con AORP
  - `cover.rs`: Generación de cover traffic
  - `discovery.rs`: Descubrimiento de nodos
- `examples/`: Ejemplos de uso
  - `basic_usage.rs`: Demo completa de modos y API
  - `identity_demo.rs`: Generación de identidades efímeras
  - `transport_demo.rs`: NAT traversal, QUIC, relays
  - `quic_server.rs` / `quic_client.rs`: Cliente/servidor QUIC
  - `relay_server.rs`: Servidor relay con autenticación efímera
- `docs/`: Documentación técnica y papers
  - `OVERVIEW.md`: Resumen técnico
  - `PAPER.md`: Borrador académico
- `TAIOR_SPEC_COMPLIANCE.md`: Mapeo completo de cumplimiento del documento Taior

## Documentación relacionada

- **Especificación y modelo de amenazas:** ver [`taior-protocol`](https://github.com/taiorproject/taior-protocol) (`README.md`, `THREAT_MODEL.md`, `PROTOCOL/`).
- **Modelo de decisión y parámetros AORP:** ver [`aorp-spec`](https://github.com/taiorproject/aorp-spec) (`SPEC.md`, `PARAMETERS.md`, `DECISION_MODEL.md`).
- **Motor de enrutamiento reusable:** [`aorp-core`](https://github.com/taiorproject/aorp-core).

## Colaboración y revisión

- Abrir issues con hallazgos, riesgos o propuestas de mejora.
- Se priorizan revisiones académicas, experimentos reproducibles y pruebas de resistencia a correlación.
- Licencias alineadas con el stack Taior: docs bajo CC BY-NC-SA 4.0; código experimental bajo AGPLv3 salvo que se indique lo contrario.

## Descargo de responsabilidad

Esta librería y sus demos son **investigación**. No ofrecen garantías de privacidad ni anonimato, y pueden contener errores. Úsese solo con expectativas de laboratorio y bajo evaluación de riesgo explícita.
