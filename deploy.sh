#!/bin/sh

set -e

REMOTE_HOST="objecthub.xyz"
REMOTE_USER="root"
REMOTE_PATH="/usr/local/goOjhub/"
BIN_PATH="target/release/openRust"

echo "Building release binary..."
cargo build --release

echo "Uploading to ${REMOTE_HOST}..."
scp -P 2243 -i "$HOME/.ssh/id_ed25519" ${BIN_PATH} ${REMOTE_USER}@${REMOTE_HOST}:${REMOTE_PATH}openRust.new

echo "Restarting service..."
ssh -p 2243 -i "$HOME/.ssh/id_ed25519" ${REMOTE_USER}@${REMOTE_HOST} "
    mv ${REMOTE_PATH}openRust.new ${REMOTE_PATH}openRust &&
    chmod +x ${REMOTE_PATH}openRust &&
    service ojhub stop &&
    service ojhub start
"

echo "Done."
