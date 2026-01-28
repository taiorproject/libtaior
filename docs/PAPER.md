# Taior / AORP: Diseño de enrutamiento amnésico y probabilístico

**Estado:** borrador académico para revisión. Investigación experimental; no usar como base de privacidad en producción.

## 1. Resumen

Taior propone un overlay con rutas emergentes hop-a-hop (AORP) que evita exponer métricas o motivos de selección. Se prioriza amnesia (borrado acelerado de estado), opacidad de heurísticas y mezcla de tráfico real con cobertura.

## 2. Problema

- Correlación de metadatos y timing en redes de baja latencia.
- Rutas deterministas o precomputadas facilitan ataques de precedencia.
- Falta de modelos reproducibles que balanceen privacidad y rendimiento con configuraciones explícitas.

## 3. Modelo de amenaza

- Adversario parcial con visibilidad en múltiples enlaces, capaz de inyectar y descartar paquetes.
- Controla un subconjunto de nodos, pero no la mayoría de guards ni todos los enlaces.
- No cubre adversario global, endpoints totalmente comprometidos ni side-channels fuera de capa de red.
- Referencia: `taior-protocol/THREAT_MODEL.md`.

## 4. Propuesta (AORP)

- Selección hop-a-hop probabilística; solo el primer guard es elegido por el emisor.
- Métricas opacas (buckets/rangos) y políticas externas (`PolicyConstraints`).
- Entropía inyectable para reproducibilidad en pruebas y aleatoriedad en despliegues.
- Amnesia: TTL corto de estado/llaves; evitar historial linkable.
- Cobertura: mezcla de paquetes dummy y jitter configurables.

## 5. Diseño de referencia

- **Motor**: `aorp-core` (Rust). Opaque scoring/selección; expone solo `NextHop`.
- **Especificación**: `aorp-spec` (DECISION_MODEL, PARAMETERS, VARIANTS).
- **Protocol-level**: `taior-protocol/PROTOCOL/` y `README.md`.

## 6. Trade-offs

- Privacidad vs latencia: más cobertura y jitter aumenta retardo.
- Amnesia vs eficiencia operativa: menos estado → más reintentos.
- Observabilidad mínima vs depuración: métricas solo agregadas.

## 7. Métricas de evaluación (sugeridas)

- Entropía de ruta (diversidad de next hops por ventana).
- Tasa de entrega vs latencia p95 en mallas de 5-15 nodos.
- Sensibilidad a adversario parcial: correlación residual entre origen y guard/destino.
- Overhead de cobertura (% dummy vs real) y coste de reintentos.

## 8. Variantes y perfiles

- **Baja latencia**: menor cobertura, diversidad media, TTL más corto.
- **Alta privacidad**: mayor cobertura, diversidad alta, jitter amplio.
- **Investigación**: reproducibilidad con `EntropySource::from_seed`, logging externo solamente.
- Parametrización en `aorp-spec/PARAMETERS.md`.

## 9. Agenda de investigación abierta

- Reputación anónima para selección de hops (tokens/zk receipts).
- Integración QUIC/WebRTC con ofuscación de tráfico.
- Evaluación formal de tamaños de set de anonimato bajo AORP.
- Modelos de economía con tickets ciegos y cuotas efímeras.

## 10. Invitación a crítica

Buscamos evaluación rigurosa de seguridad y anonimato, ataques de correlación y simulaciones reproducibles. Por favor envíe PRs o issues con metodología, scripts y parámetros. Este documento es un borrador: cite fuentes y contradiga supuestos donde aplique.

## 11. Licencia y ética

- Documentación: CC BY-NC-SA 4.0.
- Código experimental: AGPLv3 salvo que se indique lo contrario.
- Uso ético requerido: prohibido para vigilancia, seguimiento o abuso de derechos.
