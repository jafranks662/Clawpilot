# Telegram Orchestrator (Supervisor)

Configure Telegram in `~/.zeroclaw/config.toml`:

```toml
[channels_config.telegram]
bot_token = "${TELEGRAM_BOT_TOKEN}"
allowed_users = ["123456789"]
```

Enable orchestrator mode:

```toml
[orchestrator]
enabled = true
allowed_agents = ["research", "operator"]
service_prefix = "clawpilot@"
max_log_lines = 80
```

When enabled, Telegram messages are interpreted as supervisor commands.

## Commands

- `/help`
- `/status`
- `/logs <agent> [N]`
- `/start <agent>`
- `/stop <agent>`
- `/restart <agent>`
- `/run <agent> <text...>`

## Security model

- Only allowlisted `allowed_agents` can be controlled.
- Service names are built from `service_prefix + <agent>.service`.
- Agent names are strictly validated to alphanumeric, `-`, `_`.
- No arbitrary shell command execution is exposed.
