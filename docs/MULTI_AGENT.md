# Multi-agent operation on Linux

For clawpilot, the safest concurrency model is:

- **one agent = one Linux user**

This avoids collisions in per-user state under `~/.zeroclaw`.

## Create dedicated users

Examples:

- `clawpilot-research`
- `clawpilot-operator`

```bash
./scripts/create-agent-user.sh research
./scripts/create-agent-user.sh operator
```

Each user gets isolated directories:

- `/home/clawpilot-<name>/work`
- `/home/clawpilot-<name>/artifacts`
- `/home/clawpilot-<name>/logs`
- `/home/clawpilot-<name>/browser-profile`

## Run two agents concurrently

Terminal 1:

```bash
sudo -iu clawpilot-research
cd ~/work
./scripts/run-research.sh
```

Terminal 2:

```bash
sudo -iu clawpilot-operator
cd ~/work
./scripts/run-operator.sh
```

## Optional systemd template

If `systemd/clawpilot@.service` is present, you can use templated units.

```bash
sudo ./scripts/install-systemd.sh
sudo systemctl daemon-reload
sudo systemctl enable --now clawpilot@research.service
sudo systemctl enable --now clawpilot@operator.service
```

Stop/disable example:

```bash
sudo systemctl disable --now clawpilot@research.service
sudo systemctl disable --now clawpilot@operator.service
```
