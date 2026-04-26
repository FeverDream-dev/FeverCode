#!/usr/bin/env sh
set -eu
cargo build --release --bin fever
mkdir -p dist
cp target/release/fever dist/fever
ln -sf fever dist/fevercode 2>/dev/null || true
( cd dist && tar -czf "fevercode-$(uname -m)-$(uname -s | tr '[:upper:]' '[:lower:]').tar.gz" fever fevercode )
echo "Built: dist/"
ls -la dist/
