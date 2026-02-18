#!/bin/sh
set -eu

if [ "$(id -u)" -ne 0 ]; then
  echo "Please run as root (for example with sudo)."
  exit 1
fi

SERVICE_DST="/etc/systemd/system/clawpilot@.service"

if [ -f "$SERVICE_DST" ]; then
  rm -f "$SERVICE_DST"
  echo "Removed $SERVICE_DST"
else
  echo "$SERVICE_DST not present; nothing to remove."
fi

systemctl daemon-reload
