# OpenRouter + Linux Desktop Setup

This guide configures **clawpilot** as a local ZeroClaw control runtime on Linux desktops.

> Repo name is `clawpilot`; the CLI command remains `zeroclaw`.

## 1) Build and install

```bash
git clone https://github.com/YOUR_GH_USER/clawpilot.git
cd clawpilot
cargo build --release --locked
cargo install --path . --force --locked
```

## 2) Onboard with OpenRouter

Set your API key in your shell profile or an `.env` file that is **not** committed:

```bash
export OPENROUTER_API_KEY="..."
zeroclaw onboard --api-key "$OPENROUTER_API_KEY" --provider openrouter
```

The default config file location is:

```text
~/.zeroclaw/config.toml
```

## 3) Recommended safe defaults

Use supervised operation, workspace-only writes, and Docker runtime isolation.

```toml
[agent]
autonomy = "supervised"

[security]
workspace_only = true

[runtime]
kind = "docker"
network = "none"
```

If a specific task needs outbound networking, switch Docker runtime network to `bridge` for that run profile.

## 4) Operator browsing guidance

Browser automation should stay **opt-in** and requires `allowed_domains` when enabled.

```toml
[tools.browser]
enabled = true
allowed_domains = ["docs.rs", "github.com"]
```

For operator workflows on a desktop, prefer visible/headed browsing if supported by your runtime settings (for example `native_headless = false`).
