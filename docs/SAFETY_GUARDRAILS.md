# Safety guardrails for clawpilot

Use these defaults for safer local and multi-agent operation.

## Baseline

- Start in **supervised** mode.
- Keep file writes scoped with `workspace_only = true`.
- Do not grant `sudo` to runtime processes by default.

## Shell tool isolation

Prefer Docker runtime for shell execution isolation.

- Default network: `none`
- Move to `bridge` only when a workflow explicitly needs network access

## Browser guardrails

Browser automation remains opt-in.

- `tools.browser.enabled = true` only when needed
- `tools.browser.allowed_domains` must be explicitly set
- Keep allowlists minimal and task-specific

## Optional audit guidance (doc-only)

You can monitor writes under agent workspaces and artifacts with `auditd`.

Example watch targets:

- `/home/clawpilot-*/work`
- `/home/clawpilot-*/artifacts`

Example pattern (adjust for your distro/policy):

```bash
sudo auditctl -w /home/clawpilot-research/work -p wa -k clawpilot_work
sudo auditctl -w /home/clawpilot-operator/artifacts -p wa -k clawpilot_artifacts
```

Use `ausearch -k <key>` for review and clear temporary rules when finished.
