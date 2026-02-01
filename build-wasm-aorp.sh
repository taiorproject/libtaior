#!/bin/bash
set -e

echo "🔨 Compilando libtaior con AORP integrado para WASM..."

if ! command -v wasm-pack &> /dev/null; then
    echo "❌ wasm-pack no encontrado. Instalando..."
    cargo install wasm-pack
fi

echo "📦 Compilando con wasm-pack..."
wasm-pack build --target web --out-dir pkg-web --features wasm

echo "✅ Compilación WASM completada"
echo "📁 Archivos generados en: pkg-web/"
echo ""
echo "Para usar en Hush:"
echo "  npm install ./pkg-web"
echo ""
echo "Características incluidas:"
echo "  ✓ AORP decision engine integrado"
echo "  ✓ Circuitos multi-hop (3-5 saltos)"
echo "  ✓ Cifrado onion ChaCha20-Poly1305"
echo "  ✓ Cover traffic adaptativo"
echo "  ✓ Timing obfuscation"
