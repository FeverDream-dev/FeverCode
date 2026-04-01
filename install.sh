#!/bin/bash
set -euo pipefail

REPO="FeverDream-dev/FeverCode"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
SOURCE_DIR="${SOURCE_DIR:-$HOME/.local/share/fevercode}"
FEVER_BIN="$INSTALL_DIR/fever"
PATH_LINE='export PATH="$HOME/.local/bin:$PATH"'

info()  { printf '\033[1;34m[info]\033[0m  %s\n' "$*"; }
ok()    { printf '\033[1;32m[ok]\033[0m    %s\n' "$*"; }
warn()  { printf '\033[1;33m[warn]\033[0m  %s\n' "$*"; }
die()   { printf '\033[1;31m[error]\033[0m %s\n' "$*" >&2; exit 1; }

detect_distro() {
    if [[ -f /etc/os-release ]]; then
        . /etc/os-release
        case "${ID:-}" in
            ubuntu|debian|pop|linuxmint) echo "debian" ;;
            fedora)  echo "fedora" ;;
            arch|manjaro|endeavouros) echo "arch" ;;
            opensuse*|sle*) echo "opensuse" ;;
            alpine) echo "alpine" ;;
            *)
                case "${ID_LIKE:-}" in
                    *debian*) echo "debian" ;;
                    *fedora*) echo "fedora" ;;
                    *arch*)   echo "arch" ;;
                    *suse*)   echo "opensuse" ;;
                    *) echo "unknown" ;;
                esac
                ;;
        esac
    else
        echo "unknown"
    fi
}

install_pkg() {
    local distro
    distro=$(detect_distro)
    local packages=("$@")

    case "$distro" in
        debian)
            sudo apt-get update -qq && sudo DEBIAN_FRONTEND=noninteractive apt-get install -y -qq "${packages[@]}" 2>/dev/null
            ;;
        fedora)
            sudo dnf install -y "${packages[@]}" 2>/dev/null
            ;;
        arch)
            sudo pacman -Sy --noconfirm "${packages[@]}" 2>/dev/null
            ;;
        opensuse)
            sudo zypper -q install -y "${packages[@]}" 2>/dev/null
            ;;
        alpine)
            sudo apk add --no-progress "${packages[@]}" 2>/dev/null
            ;;
        *)
            die "Cannot install packages on unrecognized distro. Install manually: ${packages[*]}"
            ;;
    esac
}

ensure_curl() {
    if command -v curl >/dev/null 2>&1; then
        return 0
    fi
    if [[ "$(uname -s)" == "Darwin" ]]; then
        die "curl is required on macOS. Install Xcode Command Line Tools: xcode-select --install"
    fi
    info "Installing curl..."
    install_pkg curl
    command -v curl >/dev/null 2>&1 || die "curl install failed"
}

ensure_git() {
    if command -v git >/dev/null 2>&1; then
        return 0
    fi
    info "Installing git..."
    if [[ "$(uname -s)" == "Darwin" ]]; then
        die "git is required. Install Xcode Command Line Tools: xcode-select --install"
    fi
    install_pkg git
    command -v git >/dev/null 2>&1 || die "git install failed"
}

ensure_rust() {
    if command -v cargo >/dev/null 2>&1 && command -v rustc >/dev/null 2>&1; then
        local rust_ver
        rust_ver=$(rustc --version 2>/dev/null | grep -oP '\d+\.\d+' | head -1)
        local rust_major rust_minor
        rust_major=$(echo "$rust_ver" | cut -d. -f1)
        rust_minor=$(echo "$rust_ver" | cut -d. -f2)
        if (( rust_major > 1 || (rust_major == 1 && rust_minor >= 85) )); then
            ok "Rust $(rustc --version) detected"
            return 0
        else
            warn "Rust $(rustc --version) is too old. FeverCode requires Rust 1.85+. Updating..."
        fi
    fi

    info "Installing Rust via rustup..."
    if [[ "$(uname -s)" == "Darwin" ]]; then
        if ! command -v xcode-select >/dev/null 2>&1 || ! xcode-select -p >/dev/null 2>&1; then
            die "Xcode Command Line Tools required. Run: xcode-select --install"
        fi
    fi

    local rustup_init="/tmp/rustup-init-${USER:-root}"
    curl -fsSL https://sh.rustup.rs -o "$rustup_init"
    sh "$rustup_init" -y --default-toolchain 1.85 --profile minimal 2>/dev/null
    rm -f "$rustup_init"

    local cargo_sh="$HOME/.cargo/env"
    if [[ -f "$cargo_sh" ]]; then
        # shellcheck disable=SC1091
        . "$cargo_sh"
    fi

    command -v cargo >/dev/null 2>&1 || die "rustup install completed but cargo not found. Restart your shell and retry."
    ok "Rust $(rustc --version) installed via rustup"
}

add_to_path() {
    local line="$PATH_LINE"
    local added=false

    for rc in ~/.bashrc ~/.zshrc ~/.profile ~/.bash_profile; do
        if [[ -f "$rc" ]]; then
            if ! grep -qF 'fevercode' "$rc" 2>/dev/null && ! grep -qF '.local/bin' "$rc" 2>/dev/null; then
                printf '\n# Added by FeverCode installer\n%s\n' "$line" >> "$rc"
                added=true
            fi
        fi
    done

    if [[ "$added" == true ]]; then
        info "Added ~/.local/bin to PATH in your shell config"
    fi
}

main() {
    printf '\n\033[1m  FeverCode Installer\033[0m\n\n'

    mkdir -p "$INSTALL_DIR" "$SOURCE_DIR"

    ensure_curl
    ensure_git
    ensure_rust

    if [[ -d "$SOURCE_DIR/.git" ]]; then
        info "Updating existing source at $SOURCE_DIR..."
        git -C "$SOURCE_DIR" pull --ff-only --quiet 2>/dev/null || {
            warn "git pull failed, recloning..."
            rm -rf "$SOURCE_DIR"
        }
    fi

    if [[ ! -d "$SOURCE_DIR/.git" ]]; then
        info "Cloning FeverCode..."
        git clone --depth 1 "https://github.com/${REPO}.git" "$SOURCE_DIR" 2>/dev/null \
            || die "Failed to clone repository"
    fi

    info "Building FeverCode (this may take a few minutes)..."
    # shellcheck disable=SC1091
    [[ -f "$HOME/.cargo/env" ]] && . "$HOME/.cargo/env"
    cargo build --release --manifest-path "$SOURCE_DIR/Cargo.toml" 2>/dev/null \
        || die "Build failed. Check Rust version (need 1.85+): rustc --version"

    local src_bin="$SOURCE_DIR/target/release/fever"
    if [[ ! -f "$src_bin" ]]; then
        die "Build succeeded but binary not found at $src_bin"
    fi

    cp -f "$src_bin" "$FEVER_BIN"
    chmod +x "$FEVER_BIN"
    ok "Binary installed to $FEVER_BIN"

    add_to_path

    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        export PATH="$INSTALL_DIR:$PATH"
    fi

    local version
    version=$("$FEVER_BIN" version 2>/dev/null | head -1 || echo "unknown")
    ok "FeverCode $version installed successfully"
    printf '\n  Start with: \033[1mfever\033[0m\n\n'
}

main "$@"
