#!/bin/bash
set -e

VERSION="${VERSION:-latest}"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
USE_SUDO="${USE_SUDO:-false}"
FIREVER_REPO="FeverDream-dev/FeverCode"

echo "Fever Code Installer"
echo "===================="

# Detect architecture
ARCH=$(uname -m)
case "$ARCH" in
    x86_64)
        ARCH="x86_64"
        ;;
    aarch64|arm64)
        ARCH="aarch64"
        ;;
    *)
        echo "Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

echo "Detected architecture: $ARCH"

# Detect OS
OS=$(uname -s)
case "$OS" in
    Linux)
        OS="linux"
        ;;
    *)
        echo "Unsupported OS: $OS"
        exit 1
        ;;
esac

echo "Detected OS: $OS"

# Determine binary name
if [ "$ARCH" = "x86_64" ]; then
    BINARY_NAME="fever-${OS}-x86_64"
elif [ "$ARCH" = "aarch64" ]; then
    BINARY_NAME="fever-${OS}-aarch64"
else
    echo "Unsupported architecture for download"
    exit 1
fi

# Create install directory if it doesn't exist
if [ ! -d "$INSTALL_DIR" ]; then
    echo "Creating install directory: $INSTALL_DIR"
    mkdir -p "$INSTALL_DIR"
fi

# Download latest release
echo "Downloading Fever Code $VERSION for $OS-$ARCH..."
DOWNLOAD_URL="https://github.com/${FIREVER_REPO}/releases/download/${VERSION}/${BINARY_NAME}"

if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$DOWNLOAD_URL" -o "$INSTALL_DIR/fever"
elif command -v wget >/dev/null 2>&1; then
    wget -qO "$INSTALL_DIR/fever" "$DOWNLOAD_URL"
else
    echo "Neither curl nor wget found. Please install one of them."
    exit 1
fi

# Make binary executable
chmod +x "$INSTALL_DIR/fever"

# Create symlink for fever-code
ln -sf "$INSTALL_DIR/fever" "$INSTALL_DIR/fever-code" 2>/dev/null || true

# Check if INSTALL_DIR is in PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo ""
    echo "WARNING: $INSTALL_DIR is not in your PATH"
    echo "Add the following to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
    echo ""
    echo "export PATH=\"\$PATH:$INSTALL_DIR\""
    echo ""
    echo "Then restart your shell or run:"
    echo "source ~/.bashrc"
fi

echo ""
echo "Installation complete!"
echo ""
echo "Installed binaries:"
echo "  - $INSTALL_DIR/fever"
echo "  - $INSTALL_DIR/fever-code (symlink)"
echo ""
echo "Usage:"
echo "  fever"
echo "  fever code"
echo "  fever-code"
echo ""
echo "For more information, see: https://github.com/${FIREVER_REPO}"
