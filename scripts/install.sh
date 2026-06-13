#!/usr/bin/env sh
set -eu

REPO="${KEYFORGE_REPO:-0x3ea/keyforge}"
VERSION="${KEYFORGE_VERSION:-latest}"
INSTALL_DIR="${KEYFORGE_INSTALL_DIR:-$HOME/.local/bin}"
BIN_NAME="keyforge"

need_cmd() {
    if ! command -v "$1" >/dev/null 2>&1; then
        echo "error: required command not found: $1" >&2
        exit 1
    fi
}

detect_platform() {
    os="$(uname -s)"
    arch="$(uname -m)"

    case "$os" in
        Linux)
            case "$arch" in
                x86_64 | amd64)
                    artifact="keyforge-x86_64-unknown-linux-gnu.tar.gz"
                    ;;
                *)
                    echo "error: unsupported Linux architecture: $arch" >&2
                    exit 1
                    ;;
            esac
            ;;
        Darwin)
            case "$arch" in
                x86_64 | amd64)
                    artifact="keyforge-x86_64-apple-darwin.tar.gz"
                    ;;
                arm64 | aarch64)
                    artifact="keyforge-aarch64-apple-darwin.tar.gz"
                    ;;
                *)
                    echo "error: unsupported macOS architecture: $arch" >&2
                    exit 1
                    ;;
            esac
            ;;
        *)
            echo "error: unsupported operating system: $os" >&2
            exit 1
            ;;
    esac
}

download_url() {
    if [ "$VERSION" = "latest" ]; then
        echo "https://github.com/$REPO/releases/latest/download/$artifact"
    else
        echo "https://github.com/$REPO/releases/download/$VERSION/$artifact"
    fi
}

download_file() {
    url="$1"
    output="$2"

    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$url" -o "$output"
    elif command -v wget >/dev/null 2>&1; then
        wget -q "$url" -O "$output"
    else
        echo "error: either curl or wget is required" >&2
        exit 1
    fi
}

main() {
    need_cmd uname
    need_cmd tar
    need_cmd mktemp

    detect_platform

    existing="$INSTALL_DIR/$BIN_NAME"
    if [ -x "$existing" ]; then
        current="$("$existing" --version 2>/dev/null || echo "unknown")"
        echo "keyforge is already installed: $current"
        if [ -t 0 ]; then
            printf "Continue to update? [y/N] "
            read -r answer
            case "$answer" in
                [yY]*) ;;
                *) echo "Aborted."; exit 0 ;;
            esac
        fi
    fi

    tmp_dir="$(mktemp -d)"
    archive="$tmp_dir/$artifact"
    url="$(download_url)"

    echo "Downloading $url"
    download_file "$url" "$archive"

    echo "Installing $BIN_NAME to $INSTALL_DIR"
    mkdir -p "$INSTALL_DIR"

    tar -xzf "$archive" -C "$tmp_dir"

    if [ ! -f "$tmp_dir/$BIN_NAME" ]; then
        echo "error: archive did not contain $BIN_NAME" >&2
        exit 1
    fi

    install -m 755 "$tmp_dir/$BIN_NAME" "$INSTALL_DIR/$BIN_NAME"

    rm -rf "$tmp_dir"

    echo "keyforge installed successfully:"
    echo "  $INSTALL_DIR/$BIN_NAME"

    case ":$PATH:" in
        *":$INSTALL_DIR:"*)
            ;;
        *)
            echo
            echo "warning: $INSTALL_DIR is not in your PATH."
            echo "Add this to your shell profile:"
            echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
            ;;
    esac
}

main "$@"
