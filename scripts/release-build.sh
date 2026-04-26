#!/usr/bin/env sh
set -eu
cargo build --release --bin fever
mkdir -p dist
cp target/release/fever dist/fever
( cd dist && tar -czf fevercode-$(uname -m)-$(uname -s | tr '[:upper:]' '[:lower:]').tar.gz fever )
