# Telegram Testing Guide

## Orchestrator manual tests

1. Start orchestrator service with Telegram configured and `[orchestrator].enabled = true`.
2. Send `/status` and verify each allowlisted agent reports service state.
3. Send `/restart operator` and verify service restarts.
4. Send `/logs operator 50` and verify tail output returns.
5. Send `/run operator summarize open issues` and verify queue/result path response.
