#!/bin/bash
set -e

echo "🔨 Building libtaior for WebAssembly..."

if ! command -v wasm-pack &> /dev/null; then
    echo "❌ wasm-pack not found. Installing..."
    cargo install wasm-pack
fi

echo "📦 Building WASM package..."
wasm-pack build --target web --out-dir pkg-web --features wasm

echo "📦 Building WASM package for bundlers..."
wasm-pack build --target bundler --out-dir pkg --features wasm

echo "🔧 Fixing bundler compatibility..."
sed -i.bak 's/wasm\.__wbindgen_start();/if (typeof wasm.__wbindgen_start === "function") wasm.__wbindgen_start();/' pkg/taior.js && rm pkg/taior.js.bak

echo "✅ WASM build complete!"
echo ""
echo "📁 Output directories:"
echo "   - pkg/       (for webpack/vite/rollup)"
echo "   - pkg-web/   (for direct browser import)"
echo ""
echo "📝 To use in Hush:"
echo "   npm install ../libtaior/pkg"
