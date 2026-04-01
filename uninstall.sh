#!/bin/bash
set -euo pipefail

INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
SOURCE_DIR="${SOURCE_DIR:-$HOME/.local/share/fevercode}"
FEVER_BIN="$INSTALL_DIR/fever"

info()  { printf '\033[1;34m[info]\033[0m  %s\n' "$*"; }
warn()  { printf '\033[1;33m[warn]\033[0m  %s\n' "$*"; }
die()   { printf '\033[1;31m[error]\033[0m %s\n' "$*" >&2; exit 1; }

confirm() {
    local prompt="$1"
    local response
    printf '%s [y/N] ' "$prompt"
    read -r response
    case "$response" in
        [yY]|[yY][eE][sS]) return 0 ;;
        *) return 1 ;;
    esac
}

main() {
    printf '\n\033[1m  FeverCode Uninstaller\033[0m\n\n'

    local found_bin=false
    local found_src=false
    local found_path=false

    if [[ -f "$FEVER_BIN" ]]; then
        found_bin=true
        warn "Binary found: $FEVER_BIN"
    fi

    if [[ -d "$SOURCE_DIR" ]]; then
        found_src=true
        warn "Source directory found: $SOURCE_DIR"
    fi

    for rc in ~/.bashrc ~/.zshrc ~/.profile ~/.bash_profile; do
        if [[ -f "$rc" ]] && grep -q 'Added by FeverCode installer' "$rc" 2>/dev/null; then
            found_path=true
            warn "PATH entry found in: $rc"
        fi
    done

    if [[ "$found_bin" == false && "$found_src" == false && "$found_path" == false ]]; then
        ok "FeverCode is not installed. Nothing to do."
        exit 0
    fi

    echo ""
    confirm "Remove FeverCode?" || { info "Cancelled."; exit 0; }

    if [[ "$found_bin" == true ]]; then
        rm -f "$FEVER_BIN"
        info "Removed $FEVER_BIN"
    fi

    if [[ -f "$INSTALL_DIR/fever-code" ]]; then
        rm -f "$INSTALL_DIR/fever-code"
        info "Removed $INSTALL_DIR/fever-code"
    fi

    if [[ "$found_src" == true ]]; then
        rm -rf "$SOURCE_DIR"
        info "Removed $SOURCE_DIR"
    fi

    if [[ "$found_path" == true ]]; then
        for rc in ~/.bashrc ~/.zshrc ~/.profile ~/.bash_profile; do
            if [[ -f "$rc" ]]; then
                sed -i '/# Added by FeverCode installer/d' "$rc" 2>/dev/null || \
                sed -i '' '/# Added by FeverCode installer/d' "$rc" 2>/dev/null || true
                sed -i '/export PATH="\$HOME\/\.local\/bin:\$PATH"/d' "$rc" 2>/dev/null || \
                sed -i '' '/export PATH="\$HOME\/\.local\/bin:\$PATH"/d' "$rc" 2>/dev/null || true
            fi
        done
        info "Removed PATH entries from shell configs"
    fi

    ok "FeverCode uninstalled cleanly."
    info "Rust toolchain and cargo were preserved."
    printf '\n  Restart your shell or run: source ~/.bashrc\n\n'
}

main "$@"
