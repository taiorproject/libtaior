# Visión general de Taior (libtaior)

**Estado:** investigación experimental, no producción. Esta librería compila ideas de Taior/AORP para experimentos de enrutamiento probabilístico y amnésico. No ofrece garantías de anonimato ni ha sido auditada.

## Objetivos del repo

- Empaquetar una librería experimental de enrutamiento basada en el motor de decisión de [aorp-core](https://github.com/taiorproject/aorp-core).
- Conectar con la especificación formal y el modelo de amenazas en [taior-protocol](https://github.com/taiorproject/taior-protocol).
- Servir como espacio para demos reproducibles y experimentos controlados.
- Documentar de forma explícita los límites y riesgos; invitar a revisión académica.

## Cómo encaja con el stack Taior

- **Especificación y diseño** → `taior-protocol` (conceptos, threat model, parámetros).
- **Modelo de decisión abstracto** → `aorp-spec` (pseudocódigo, variantes, parámetros, arquitectura).
- **Motor de enrutamiento reusable** → `aorp-core` (Rust, opaco, sin métricas reveladas).
- **Integración y demos** → `libtaior` (este repo), ensamblando el motor con ejemplos.

## Expectativas y límites

- Uso en laboratorio, entornos controlados y simulaciones. No usar en producción ni con adversarios avanzados.
- Los parámetros y perfiles aquí documentados son guías para experimentos, no configuraciones recomendadas para despliegues reales.
- No se incluye cifrado de transporte ni gestión de identidad; asume capas externas.

## Conceptos clave (referencias cruzadas)

- **AORP**: enrutamiento hop-a-hop con selección probabilística y rutas emergentes. Ver `taior-protocol/PROTOCOL/` y `aorp-spec/DECISION_MODEL.md`.
- **Amnesia**: minimizar retención de estado y rotar claves. Ver `taior-protocol/THREAT_MODEL.md` y `taior-protocol/README.md`.
- **Cobertura (cover traffic)**: mezcla de paquetes reales y dummy; parámetros en `taior-protocol/PARAMETERS.md` y `aorp-spec/PARAMETERS.md`.
- **Métricas opacas**: buckets de latencia y rangos de ancho de banda, nunca valores crudos. Ver `aorp-spec/PSEUDOCODE.md`.

## Licencias

- Documentación: CC BY-NC-SA 4.0.
- Código experimental y demos: AGPLv3 salvo que se indique lo contrario.

## Colaboración y crítica

- Buscamos revisión académica, replicación de simulaciones y ataques experimentales.
- Abra issues con hallazgos, riesgos o propuestas; se priorizan reproducciones y métricas claras.
- No introducir telemetría identificable ni dependencias cerradas.

## Descargo de responsabilidad

Taior/libtaior es investigación. No garantiza privacidad, anonimato ni robustez ante adversarios determinados. Úsese solo bajo evaluación de riesgo explícita y con expectativas de laboratorio.
