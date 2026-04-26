#!/usr/bin/env sh
set -eu

REPO="${FEVERCODE_REPO:-YOUR_ORG/fevercode}"
BIN_DIR="${FEVERCODE_BIN_DIR:-$HOME/.local/bin}"
VERSION="${FEVERCODE_VERSION:-latest}"

printf '%s\n' "𓂀 FeverCode installer"
printf '%s\n' "Target bin dir: $BIN_DIR"
mkdir -p "$BIN_DIR"

need() {
  command -v "$1" >/dev/null 2>&1 || return 1
}

if need cargo && [ -f Cargo.toml ]; then
  printf '%s\n' "Local Cargo project detected. Installing from current directory..."
  cargo install --path . --locked --force
  printf '%s\n' "Installed fever and fevercode via cargo."
  exit 0
fi

OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"
case "$ARCH" in
  x86_64|amd64) ARCH="x86_64" ;;
  arm64|aarch64) ARCH="aarch64" ;;
  *) printf '%s\n' "Unsupported architecture: $ARCH" >&2; exit 1 ;;
esac

case "$OS" in
  darwin) OS="apple-darwin" ;;
  linux) OS="unknown-linux-gnu" ;;
  *) printf '%s\n' "Unsupported OS for this MVP: $OS" >&2; exit 1 ;;
esac

ASSET="fevercode-${ARCH}-${OS}.tar.gz"
if [ "$VERSION" = "latest" ]; then
  URL="https://github.com/${REPO}/releases/latest/download/${ASSET}"
else
  URL="https://github.com/${REPO}/releases/download/${VERSION}/${ASSET}"
fi

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

printf '%s\n' "Downloading $URL"
if need curl; then
  curl -fsSL "$URL" -o "$TMP/$ASSET"
elif need wget; then
  wget -q "$URL" -O "$TMP/$ASSET"
else
  printf '%s\n' "Install curl or wget first." >&2
  exit 1
fi

tar -xzf "$TMP/$ASSET" -C "$TMP"
install -m 755 "$TMP/fever" "$BIN_DIR/fever"
ln -sf "$BIN_DIR/fever" "$BIN_DIR/fevercode"

printf '%s\n' "Installed fever to $BIN_DIR/fever"
printf '%s\n' "Run: fever doctor"
