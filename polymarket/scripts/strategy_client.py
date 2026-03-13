#!/usr/bin/env python3
"""
🎯 Polymarket Strategy Client (Python)

Cliente gRPC en Python para comunicarse con el servicio PolymarketBot en Rust.
Permite prototipar estrategias rápidamente en Python mientras la ejecución
permanece en Rust (alta velocidad).

Uso:
    python3 polymarket/scripts/strategy_client.py --action markets
    python3 polymarket/scripts/strategy_client.py --action positions
"""

import argparse
import sys
import os

# Agregar path para importar proto generados
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', '..', 'intelligence', 'scripts'))


def list_markets(limit: int = 20):
    """Lista mercados de predicción disponibles."""
    print(f"📊 Consultando {limit} mercados de Polymarket...")
    print("=" * 60)
    # TODO: Implementar llamada gRPC cuando el servidor esté activo
    print("  ⚠️  Servidor gRPC no conectado.")
    print("  Ejecutar: cargo run -p polymarket_bot -- serve")
    print("=" * 60)


def show_positions():
    """Muestra posiciones abiertas."""
    print("📋 Consultando posiciones abiertas...")
    print("=" * 60)
    # TODO: Implementar llamada gRPC
    print("  ⚠️  Servidor gRPC no conectado.")
    print("=" * 60)


def main():
    parser = argparse.ArgumentParser(
        description="🎯 Polymarket Strategy Client"
    )
    parser.add_argument(
        "--action",
        choices=["markets", "positions"],
        required=True,
        help="Acción a ejecutar"
    )
    parser.add_argument(
        "--limit",
        type=int,
        default=20,
        help="Límite de mercados (default: 20)"
    )

    args = parser.parse_args()

    if args.action == "markets":
        list_markets(args.limit)
    elif args.action == "positions":
        show_positions()


if __name__ == "__main__":
    main()
