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

echo "🛠️  Setting scoped package metadata..."
node -e "const fs=require('fs');const p='pkg/package.json';const pkg=JSON.parse(fs.readFileSync(p,'utf8'));pkg.name='@taiorproject/taior';pkg.publishConfig={registry:'https://npm.pkg.github.com'};fs.writeFileSync(p,JSON.stringify(pkg,null,2)+'\n');"

echo "✅ WASM build complete!"
echo ""
echo "📁 Output directories:"
echo "   - pkg/       (for webpack/vite/rollup)"
echo "   - pkg-web/   (for direct browser import)"
echo ""
echo "📝 To use in Hush:"
echo "   npm install ../libtaior/pkg"
