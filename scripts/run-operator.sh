#!/bin/sh
set -eu

if [ -f ./.env ]; then
  set -a
  # shellcheck disable=SC1091
  . ./.env
  set +a
fi

CONFIG_FILE="${HOME}/.zeroclaw/config.toml"

if [ ! -f "$CONFIG_FILE" ]; then
  echo "Missing ${CONFIG_FILE}."
  echo "Run: zeroclaw onboard --api-key \"\$OPENROUTER_API_KEY\" --provider openrouter"
  exit 1
fi

echo "Reminder: run supervised and set tools.browser.allowed_domains before enabling browser automation."
exec zeroclaw agent
