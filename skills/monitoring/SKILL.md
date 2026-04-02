---
name: latency-monitoring
description: Monitoriza p99.9 de latencia del motor HFT en GCP.
version: 1.0.0
---

# SKILL: Latency Monitoring

Este skill permite realizar una auditoría de rendimiento en tiempo real del motor "The Chassis".

## Instrucciones para Agentes de IA

1. **Contexto**: El motor corre en un contenedor Docker en la instancia `solana-bot-v1` (GCP).
2. **Acción**: Ejecutar el comando de monitoreo de logs filtrando por latencia.
3. **Verificación**: Comparar el jitter reportado contra el umbral de <100µs.

### Comandos Atómicos
- `gcloud compute ssh solana-bot-v1 --command "sudo docker logs --tail 100 solana_trading_bot | grep -i latency"`

## Casos de Uso
- Detectar degradación de red en Raydium.
- Validar optimizaciones de zero-alloc en el executor.
