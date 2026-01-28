# Demos de libtaior (experimentales)

**Advertencia:** demos solo para laboratorio. Requieren `Rust` y el motor `aorp-core`. No proveen anonimato fuerte ni seguridad produccional.

## Prerrequisitos

- Rust estable (`rustup`) y `cargo`.
- Acceso a internet para obtener `aorp-core` desde Git (rama `main`).

## Estructura

- `Cargo.toml`: manifiesto de demo con dependencia a `aorp-core`.
- `src/main.rs`: ejemplo mínimo de selección de siguiente salto con métricas opacas.

## Ejecutar demo

```bash
cargo run --manifest-path demos/Cargo.toml
```

Salida esperada (variará por aleatoriedad):

```text
Próximo salto: NextHop(NeighborId("n2"))
```

## Qué hace la demo

- Construye un `NeighborSet` con nodos ficticios.
- Define métricas en buckets (latencia, ancho de banda) sin exponer valores crudos.
- Aplica políticas (`PolicyConstraints`) para diversidad y pesos de latencia/ancho de banda.
- Usa `EntropySource::secure_random()` para seleccionar el siguiente salto de forma probabilística.

## Ajustes sugeridos

- Cambiar `EntropySource::from_seed(42)` para reproducibilidad en pruebas.
- Modificar `latency_weight`/`bandwidth_weight` y `DiversityLevel` para explorar sensibilidad.
- Añadir más vecinos o buckets en `MetricView::builder()`.

## Notas de seguridad

- No registra puntuaciones internas; el motor es opaco por diseño.
- No incorpora cifrado de transporte ni autenticación de nodos; asume capas externas.
- Los parámetros aquí no son recomendaciones de producción.

## Referencias

- Especificación y modelo de amenazas: `taior-protocol`.
- Modelo de decisión y parámetros: `aorp-spec`.
- Motor de enrutamiento: `aorp-core`.
