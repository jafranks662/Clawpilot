# Debian packaging

Build package:

```bash
cargo install cargo-deb
cargo deb
```

Install package:

```bash
sudo apt install ./target/debian/*.deb
```

Environment files live in `/etc/clawpilot/`.

- `/etc/clawpilot/orchestrator.env`
- `/etc/clawpilot/research.env`
- `/etc/clawpilot/operator.env`
