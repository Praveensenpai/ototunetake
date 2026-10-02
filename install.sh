#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

BIN_NAME="ototunetake"
INSTALL_DIR="${HOME}/.local/bin"

mkdir -p "${INSTALL_DIR}"

if command -v cargo >/dev/null 2>&1; then
    echo "==> Building ${BIN_NAME} with cargo..."
    cargo build --release
    install -m 755 "target/release/${BIN_NAME}" "${INSTALL_DIR}/${BIN_NAME}"
    echo "==> Installed ${BIN_NAME} to ${INSTALL_DIR}/${BIN_NAME}"
else
    echo "Error: cargo is required to build ${BIN_NAME}" >&2
    exit 1
fi
