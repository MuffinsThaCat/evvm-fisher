#!/bin/bash
# Build script for Fisher Relayer

set -e

echo "🔨 Building Fisher Relayer..."

# Build native binary
echo "📦 Building native binary..."
cargo build --release

# Build WASM for Enarx
echo "🌐 Building WASM for Enarx..."
cargo build --target wasm32-wasi --release

# Optimize WASM
if command -v wasm-opt &> /dev/null; then
    echo "⚡ Optimizing WASM..."
    wasm-opt -Oz --enable-bulk-memory \
        target/wasm32-wasi/release/fisher_relayer.wasm \
        -o target/wasm32-wasi/release/fisher_relayer_optimized.wasm
    
    echo "✅ Optimized WASM created"
fi

echo "✨ Build complete!"
echo ""
echo "Native binary: ./target/release/fisher-relayer"
echo "WASM module:   ./target/wasm32-wasi/release/fisher_relayer.wasm"
echo ""
echo "To run in Enarx TDX:"
echo "  enarx run --backend tdx target/wasm32-wasi/release/fisher_relayer.wasm"
