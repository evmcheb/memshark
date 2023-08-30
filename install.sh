#!/usr/bin/env bash
set -e

# Base and Shark directories
BASE_DIR=${XDG_CONFIG_HOME:-$HOME}
SHARK_DIR=${SHARK_DIR:-"$BASE_DIR/.shark"}

# Detect shell and choose profile
case $SHELL in
  */zsh)  PROFILE=${ZDOTDIR:-"$HOME"}/.zshenv ;;
  */bash) PROFILE="$HOME/.bashrc" ;;
  */fish) PROFILE="$HOME/.config/fish/config.fish" ;;
  */ash)  PROFILE="$HOME/.profile" ;;
  *) echo "shark install: could not detect shell, manually add ${SHARK_DIR} to your PATH."; exit 1 ;;
esac

# Detect architecture and OS
ARCH=$(uname -m)
OS=$(uname -s)
case $ARCH in
  x86_64) ARCH="x86_64" ;;
  aarch64) ARCH="aarch64" ;;
  *) echo "Unsupported architecture"; exit 1 ;;
esac
case $OS in
  Darwin) OS="apple-darwin" ;;
  Linux) OS="unknown-linux-gnu" ;;
  *) echo "Unsupported OS"; exit 1 ;;
esac

# Download and extract binary
URL="https://github.com/evmcheb/shark/releases/download/latest/shark-${ARCH}-${OS}.tar.gz"
mkdir -p "$SHARK_DIR"
curl -L "$URL" | tar xz -C "$SHARK_DIR"

# Make the binary executable
chmod +x "$SHARK_DIR/shark"

# Update PATH if necessary
if [[ ":$PATH:" != *":${SHARK_DIR}:"* ]]; then
  echo "export PATH=\"\$PATH:$SHARK_DIR\"" >> "$PROFILE"
fi

echo "shark installed successfully."
