#!/bin/sh
set -eu

if [ "$(id -u)" -ne 0 ]; then
  echo "Please run as root (for example with sudo)."
  exit 1
fi

if [ "$#" -ne 1 ]; then
  echo "Usage: $0 <name>"
  echo "Example: $0 research"
  exit 1
fi

NAME="$1"
USER="clawpilot-${NAME}"
HOME_DIR="/home/${USER}"

if ! command -v useradd >/dev/null 2>&1; then
  echo "useradd command not found. This script targets Debian-based systems."
  exit 1
fi

if id "$USER" >/dev/null 2>&1; then
  echo "User ${USER} already exists."
else
  useradd --create-home --shell /bin/bash "$USER"
  echo "Created user ${USER}."
fi

install -d -m 0750 -o "$USER" -g "$USER" \
  "$HOME_DIR/work" \
  "$HOME_DIR/artifacts" \
  "$HOME_DIR/logs" \
  "$HOME_DIR/browser-profile"

if command -v getent >/dev/null 2>&1 && getent group docker >/dev/null 2>&1; then
  usermod -aG docker "$USER"
  echo "Added ${USER} to docker group."
else
  echo "docker group not found; skipping docker group membership."
fi

echo "Done. No secrets were created or stored by this script."
