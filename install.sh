#!/usr/bin/env sh
set -eu

# FeverCode installer — downloads the latest release for your platform.
# Usage: curl --proto '=https' --tlsv1.2 -LsSf https://github.com/FeverDream-dev/FeverCode/releases/latest/download/fever-installer.sh | sh
#
# Environment variables:
#   FEVER_INSTALL_DIR  — override install directory (default: $HOME/.local/bin)
#   FEVER_VERSION      — specific version to install (default: latest)
#   FEVER_NO_MODIFY_PATH — set to 1 to skip PATH modification

REPO="FeverDream-dev/FeverCode"
VERSION="${FEVER_VERSION:-latest}"
BIN_DIR="${FEVER_INSTALL_DIR:-$HOME/.local/bin}"
NO_MODIFY_PATH="${FEVER_NO_MODIFY_PATH:-}"

printf '%s\n' "𓂀 FeverCode installer"
printf '%s\n' "Version: $VERSION"
printf '%s\n' "Target: $BIN_DIR"

need() {
    command -v "$1" >/dev/null 2>&1
}

# Source fallback: if cargo is available and we're in a FeverCode checkout
if need cargo && [ -f Cargo.toml ] && [ -f src/main.rs ]; then
    printf '%s\n' "Local Cargo project detected. Installing from source..."
    cargo install --path . --locked --force
    printf '%s\n' "Installed fever and fevercode via cargo."
    exit 0
fi

# Detect platform
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$ARCH" in
    x86_64|amd64) ARCH="x86_64" ;;
    arm64|aarch64) ARCH="aarch64" ;;
    *) printf '%s\n' "Unsupported architecture: $ARCH" >&2; exit 1 ;;
esac

case "$OS" in
    darwin) TARGET="${ARCH}-apple-darwin" ;;
    linux) TARGET="${ARCH}-unknown-linux-gnu" ;;
    *) printf '%s\n' "Unsupported OS: $OS. Use source install: cargo install --git https://github.com/${REPO} fever" >&2; exit 1 ;;
esac

ASSET="fevercode-${TARGET}.tar.gz"
if [ "$VERSION" = "latest" ]; then
    URL="https://github.com/${REPO}/releases/latest/download/${ASSET}"
else
    URL="https://github.com/${REPO}/releases/download/${VERSION}/${ASSET}"
fi

# Download
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

printf '%s\n' "Downloading $URL"
download_failed() {
    printf '%s\n' "Download failed. No release available yet?" >&2
    if need cargo; then
        printf '%s\n' "Installing from source instead..." >&2
        cargo install --git "https://github.com/${REPO}" fevercode --locked --bin fever --force || {
            printf '%s\n' "Source install failed." >&2
            printf '%s\n' "Try manually:" >&2
            printf '%s\n' "  cargo install --git https://github.com/${REPO} fevercode --bin fever" >&2
            exit 1
        }
        printf '%s\n' "Installed fever and fevercode via cargo."
        exit 0
    fi
    printf '%s\n' "Install from source:" >&2
    printf '%s\n' "  cargo install --git https://github.com/${REPO} fevercode --bin fever" >&2
    exit 1
}

if need curl; then
    curl -fsSL "$URL" -o "$TMP/$ASSET" || download_failed
elif need wget; then
    wget -q "$URL" -O "$TMP/$ASSET" || download_failed
else
    printf '%s\n' "Install curl or wget first." >&2
    exit 1
fi

# Extract and install
tar -xzf "$TMP/$ASSET" -C "$TMP"
mkdir -p "$BIN_DIR"
install -m 755 "$TMP/fever" "$BIN_DIR/fever"
ln -sf "$BIN_DIR/fever" "$BIN_DIR/fevercode" 2>/dev/null || true

printf '%s\n' ""
printf '%s\n' "Installed fever to $BIN_DIR/fever"
printf '%s\n' "Linked fevercode -> fever"

# PATH check
case ":${PATH}:" in
    *":${BIN_DIR}:"*) ;;
    *)
        if [ -z "$NO_MODIFY_PATH" ]; then
            SHELL_RC=""
            if [ -f "$HOME/.bashrc" ]; then SHELL_RC="$HOME/.bashrc"
            elif [ -f "$HOME/.zshrc" ]; then SHELL_RC="$HOME/.zshrc"
            fi
            if [ -n "$SHELL_RC" ]; then
                printf '\nexport PATH="%s:$PATH"\n' "$BIN_DIR" >> "$SHELL_RC"
                printf '%s\n' "Added $BIN_DIR to PATH in $SHELL_RC"
                printf '%s\n' "Run: source $SHELL_RC  (or open a new terminal)"
            else
                printf '%s\n' "Add $BIN_DIR to your PATH:"
                printf '%s\n' "  export PATH=\"$BIN_DIR:\$PATH\""
            fi
        fi
        ;;
esac

printf '%s\n' ""
printf '%s\n' "Run: fever doctor"
