use anyhow::{bail, Context, Result};
use tokio::process::Command;

#[derive(Debug, Clone)]
pub struct SystemdController;

impl SystemdController {
    pub async fn is_active(&self, service: &str) -> Result<String> {
        run_command("systemctl", ["is-active", service]).await
    }

    pub async fn start(&self, service: &str) -> Result<String> {
        run_command("systemctl", ["start", service]).await
    }

    pub async fn stop(&self, service: &str) -> Result<String> {
        run_command("systemctl", ["stop", service]).await
    }

    pub async fn restart(&self, service: &str) -> Result<String> {
        run_command("systemctl", ["restart", service]).await
    }

    pub async fn logs(&self, service: &str, lines: usize) -> Result<String> {
        run_command(
            "journalctl",
            ["-u", service, "-n", &lines.to_string(), "--no-pager"],
        )
        .await
    }
}

async fn run_command<const N: usize>(program: &str, args: [&str; N]) -> Result<String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .await
        .with_context(|| format!("failed to run {program}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("{program} failed: {}", stderr.trim());
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
