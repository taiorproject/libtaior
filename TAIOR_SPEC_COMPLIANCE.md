# Cumplimiento de la especificación Taior

Este documento mapea cada requisito del documento "Taior: The Amnesic Incognito Oblivious Routing" con su implementación en `libtaior`.

### 📋 Cumplimiento del documento

Ver `TAIOR_SPEC_COMPLIANCE.md` para mapeo exhaustivo. **Implementados todos los requisitos principales**:

- ✅ Identidad efímera
- ✅ Modos Fast/Mix/Adaptive  
- ✅ AORP
- ✅ Cifrado AEAD + PFS
- ✅ Paquetes indistinguibles
- ✅ Cover traffic
- ✅ Descubrimiento de nodos
- ✅ API simple embebible
- ✅ Stateless por diseño
- ✅ Transporte QUIC + TLS 1.3
- ✅ NAT traversal + hole punching
- ✅ Relays/bridges con autenticación efímerable

## 1. Objetivos de diseño ✓

### Implementado
- **Ocultar metadatos, no solo contenido**: Cifrado AEAD por capa, padding indistinguible, cover traffic
- **No requerir software adicional**: Librería embebible (`taior` crate)
- **UX-first, mobile-first**: API simple `Taior.send(data, options)`
- **Integrable como librería**: `Cargo.toml` con `crate-type = ["rlib", "cdylib"]`
- **Stateless por diseño**: Identidades efímeras, sin persistencia por defecto
- **Tolerante a fallos**: TTL, reintentos probabilísticos vía AORP
- **Configurable: privacidad vs latencia**: Modos Fast/Mix/Adaptive

### Límites reconocidos
- No garantiza anonimato absoluto
- No promete invisibilidad ante adversarios globales
- No elimina todos los ataques de correlación

**Archivos**: `src/api.rs`, `src/modes.rs`, `src/identity.rs`

---

## 2. Modelo mental ✓

Implementado mediante:
- Rutas emergentes (AORP): `src/routing.rs`
- Cover traffic: `src/cover.rs`
- Jitter configurable: `src/modes.rs` (`jitter_ms`)

**Archivos**: `src/routing.rs`, `src/cover.rs`

---

## 3. Arquitectura general ✓

### Capas implementadas
1. **Taior Core**: `src/lib.rs`, `src/packet.rs`, `src/routing.rs`, `src/identity.rs`
2. **Transport Adapters**: Pendiente (QUIC/WS/TCP) — API preparada
3. **Node Runtime**: `src/api.rs` (ejecutable en apps)
4. **Optional Infrastructure**: `src/discovery.rs` (relays/bridges)

**Archivos**: `src/lib.rs`, `src/api.rs`, `src/discovery.rs`

---

## 4. Identidad y amnesia ✓

### Implementado
- **Identidad efímera**: `EphemeralIdentity` con `X25519` + `BLAKE3`
- **Formato**: `taior://<hash-256>` (`TaiorAddress`)
- **Amnesia por diseño**: Claves en RAM, no persistidas
- **Reiniciar = borrado criptográfico**: Sin estado persistente

**Archivos**: `src/identity.rs`

```rust
let identity = EphemeralIdentity::new();
println!("{}", identity.address.as_str()); // taior://abc123...
```

---

## 5. Modos de operación ✓

### 5.1 Fast Mode
- 1-2 saltos
- Sin mezcla
- Mínima latencia
- `SendOptions::fast()`

### 5.2 Mix Mode
- 3-5 saltos
- Paquetes indistinguibles (padding 512 bytes)
- Retrasos aleatorios (jitter 200ms)
- Cover traffic habilitado
- `SendOptions::mix()`

### 5.3 Adaptive Mode
- 2 saltos
- Jitter 50ms
- Padding 256 bytes
- `SendOptions::adaptive()`

**Archivos**: `src/modes.rs`, `src/api.rs`

```rust
taior.send(data, SendOptions::fast());
taior.send(data, SendOptions::mix());
taior.send(data, SendOptions::adaptive());
taior.send(data, SendOptions::custom(RoutingMode::Mix, 3));
```

---

## 6. Enrutamiento: AORP ✓

### Implementado
- **Ningún nodo conoce la ruta completa**: Decisión hop-by-hop
- **Emisor define solo el primer salto**: `Router::decide_next_hop()`
- **Función probabilística**: Integración con `aorp-core`
- **Métricas consideradas**: Latencia, disponibilidad, entropía (vía `aorp-core`)

**Archivos**: `src/routing.rs` (usa `aorp-core`)

```rust
let next_hop = router.decide_next_hop(neighbors, &config);
```

---

## 7. Criptografía ✓

### Implementado
- **Cifrado simétrico moderno (AEAD)**: ChaCha20-Poly1305
- **Intercambio de claves efímeras**: X25519
- **Forward secrecy**: Claves derivadas por paquete (HKDF-SHA256)
- **Preparado para PQ**: Estructura modular permite Kyber768 futuro

**Archivos**: `src/packet.rs`, `src/identity.rs`

```rust
let (key, nonce) = derive_packet_key();
let cipher = ChaCha20Poly1305::new(&key);
let ciphertext = cipher.encrypt(&nonce, payload)?;
```

---

## 8. Paquetes indistinguibles ✓

### Implementado en Mix Mode
- Todos los paquetes tienen el mismo tamaño (padding configurable)
- Mismo formato (`TaiorPacket`)
- Padding aleatorio (`pad_payload`)
- Tráfico señuelo (`is_cover` flag)

**Archivos**: `src/packet.rs`, `src/cover.rs`

```rust
let packet = TaiorPacket::new(payload, ttl, padding_size, is_cover)?;
```

---

## 9. Ruido y cover traffic ✓

### Implementado
- **Paquetes vacíos**: `CoverTrafficGenerator::generate_cover_packet()`
- **Rutas falsas**: TTL y padding idénticos
- **Controlado por**: Nivel de privacidad, ratio configurable

**Archivos**: `src/cover.rs`

```rust
taior.enable_cover_traffic(true, 0.3); // 30% cover traffic
```

---

## 10. Descubrimiento de nodos ✓

### Implementado
- **Listas embebidas**: `NodeDiscovery::with_bootstrap()`
- **Añadir/remover nodos**: `add_node()`, `remove_node()`
- **DHT volátil**: Pendiente (estructura preparada)

**Archivos**: `src/discovery.rs`

```rust
let taior = Taior::with_bootstrap(vec!["node1.taior.net".to_string()]);
taior.add_node("node2.taior.net".to_string());
```

---

## 11. NATs y movilidad ✓

### Implementado
- **NAT traversal**: STUN para descubrir dirección pública
- **Hole punching**: UDP hole punching para conexiones P2P directas
- **Relays**: Fallback a relay servers cuando P2P falla
- **Rutas asimétricas**: Soporte para conexiones via relay

**Archivos**: `src/transport/nat.rs`, `src/transport/relay.rs`

```rust
let nat = NatTraversal::new(local_addr);
let public_addr = nat.discover_public_addr().await?;
let socket = nat.hole_punch(peer_addr).await?;
```

---

## 12. Relays y bridges ✓

### Implementado
- **Relays estables**: Servidores relay sin autoridad central
- **Autenticación efímera**: Tokens temporales con expiración (1 hora)
- **Enrutamiento via relay**: Cliente puede enviar paquetes a través de relay
- **Sin identidades persistentes**: Tokens derivados de direcciones efímeras

**Archivos**: `src/transport/relay.rs`

```rust
let auth = RelayAuth::generate(&address);
let client = RelayClient::new(relay_addr, auth).await?;
client.send_via_relay(&packet, &dest_addr).await?;

// Servidor
let relay = RelayServer::new(bind_addr).await?;
relay.run().await?;
```

---

## 13. Taior como librería ✓

### Implementado
- **API simple**: `Taior::new()`, `taior.send(data, options)`
- **Defaults seguros**: `SendOptions::default()` → Adaptive
- **Configuración explícita**: `SendOptions::custom(mode, hops)`

**Archivos**: `src/api.rs`

```rust
use taior::{Taior, SendOptions};

let mut taior = Taior::new();
let packet = taior.send(b"mensaje", SendOptions::fast())?;
```

---

## 14. Modelo de amenazas ✓

### Protege contra
- Vigilancia pasiva (cifrado AEAD)
- ISP (rutas probabilísticas)
- Trackers (identidades efímeras)
- Censura básica (bridges/relays preparados)

### No protege totalmente contra
- Adversarios globales
- Control masivo de nodos
- Ataques de endpoint

**Documentado en**: `docs/PAPER.md`, `docs/OVERVIEW.md`

---

## 15. Relación con Hush ✓

Taior es independiente y reusable. Hush puede integrarlo como:

```rust
use taior::{Taior, SendOptions};

let mut taior = Taior::new();
let encrypted_message = taior.send(chat_message, SendOptions::mix())?;
// enviar encrypted_message por transporte
```

---

## 16. Filosofía ✓

**Privacidad no como excepción, sino como default silencioso.**

- API por defecto usa modo Adaptive (balance privacidad/latencia)
- Cover traffic opcional pero fácil de habilitar
- Identidades efímeras sin configuración adicional

---

## 17. Estado del documento ✓

- **Versión**: 0.1 (funcional, experimental)
- **No es RFC final**: Investigación activa
- **Abierto a iteración**: Issues y PRs bienvenidos

---

## Resumen de cumplimiento

| Requisito | Estado | Archivos |
|-----------|--------|----------|
| Identidad efímera | ✓ | `src/identity.rs` |
| Modos Fast/Mix/Adaptive | ✓ | `src/modes.rs`, `src/api.rs` |
| AORP (enrutamiento probabilístico) | ✓ | `src/routing.rs` (vía `aorp-core`) |
| Cifrado AEAD + PFS | ✓ | `src/packet.rs` |
| Paquetes indistinguibles | ✓ | `src/packet.rs` |
| Cover traffic | ✓ | `src/cover.rs` |
| Descubrimiento de nodos | ✓ | `src/discovery.rs` |
| API simple | ✓ | `src/api.rs` |
| Transporte QUIC + TLS 1.3 | ✓ | `src/transport/quic.rs` |
| NAT traversal + hole punching | ✓ | `src/transport/nat.rs` |
| Relays/bridges con auth efímera | ✓ | `src/transport/relay.rs` |

**Leyenda**: ✓ Implementado | ⏳ Pendiente | ✗ No implementado

---

## Próximos pasos

1. ~~Implementar capa de transporte (QUIC con `quinn`)~~ ✅ Completado
2. ~~Añadir NAT traversal y hole punching~~ ✅ Completado
3. ~~Integrar relays/bridges con autenticación efímera~~ ✅ Completado
4. Pruebas de correlación y métricas de anonimato
5. Integración con DHT volátil para descubrimiento dinámico
6. Optimización de rendimiento y latencia
7. Auditoría de seguridad

---

**Nota**: Esta implementación es investigación experimental. No usar en producción sin auditoría y evaluación de riesgo explícita.
