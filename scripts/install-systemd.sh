#!/bin/sh
set -eu

if [ "$(id -u)" -ne 0 ]; then
  echo "Please run as root (for example with sudo)."
  exit 1
fi

SERVICE_SRC="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)/systemd/clawpilot@.service"
SERVICE_DST="/etc/systemd/system/clawpilot@.service"

install -D -m 0644 "$SERVICE_SRC" "$SERVICE_DST"
install -d -m 0755 /etc/clawpilot

systemctl daemon-reload

echo "Installed $SERVICE_DST"
echo "Create per-instance env files under /etc/clawpilot/<name>.env as needed."
