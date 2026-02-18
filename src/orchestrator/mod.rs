pub mod commands;
pub mod systemd;

use crate::config::OrchestratorConfig;
use anyhow::{bail, Context, Result};
use chrono::Utc;
use commands::{parse_command, OrchestratorCommand};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::time::{sleep, Duration};
use uuid::Uuid;

use self::systemd::SystemdController;

const DEFAULT_QUEUE_ROOT: &str = "/var/lib/clawpilot/queue";
const DEFAULT_RESULTS_ROOT: &str = "/var/lib/clawpilot/results";
const JOB_TIMEOUT_SECONDS: u64 = 120;

#[derive(Debug, Clone)]
pub struct Orchestrator {
    config: OrchestratorConfig,
    systemd: SystemdController,
    queue_root: PathBuf,
    results_root: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentJob {
    pub id: String,
    pub agent: String,
    pub text: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentJobResult {
    pub id: String,
    pub agent: String,
    pub status: String,
    pub summary: String,
    pub created_at: String,
    pub finished_at: String,
}

impl Orchestrator {
    pub fn from_config(config: OrchestratorConfig) -> Self {
        Self {
            config,
            systemd: SystemdController,
            queue_root: PathBuf::from(DEFAULT_QUEUE_ROOT),
            results_root: PathBuf::from(DEFAULT_RESULTS_ROOT),
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    pub async fn handle_message(&self, message: &str) -> Result<String> {
        let cmd = parse_command(message).map_err(|e| anyhow::anyhow!("{e}. Try /help"))?;
        match cmd {
            OrchestratorCommand::Help => Ok(self.help_text()),
            OrchestratorCommand::Status => self.status().await,
            OrchestratorCommand::Logs { agent, lines } => self.logs(&agent, lines).await,
            OrchestratorCommand::Restart { agent } => self.act(&agent, "restart").await,
            OrchestratorCommand::Start { agent } => self.act(&agent, "start").await,
            OrchestratorCommand::Stop { agent } => self.act(&agent, "stop").await,
            OrchestratorCommand::Run { agent, text } => self.run_job(&agent, &text).await,
        }
    }

    fn help_text(&self) -> String {
        format!(
            "Orchestrator commands:\n/help\n/status\n/logs <agent> [N]\n/start <agent>\n/stop <agent>\n/restart <agent>\n/run <agent> <text...>\n\nAllowed agents: {}",
            self.config.allowed_agents.join(", ")
        )
    }

    async fn status(&self) -> Result<String> {
        let mut out = String::from("Orchestrator service status:\n");
        for agent in &self.config.allowed_agents {
            let service = self.service_name(agent)?;
            let active = self
                .systemd
                .is_active(&service)
                .await
                .unwrap_or_else(|e| format!("error: {e}"));
            let logs = self
                .systemd
                .logs(&service, 2)
                .await
                .unwrap_or_else(|e| format!("log error: {e}"));
            out.push_str(&format!("\n- {agent}: {active}\n{logs}\n"));
        }
        Ok(out)
    }

    async fn logs(&self, agent: &str, lines: Option<usize>) -> Result<String> {
        let service = self.service_name(agent)?;
        let requested = lines.unwrap_or(self.config.max_log_lines);
        let safe_lines = requested.min(self.config.max_log_lines).max(1);
        let output = self.systemd.logs(&service, safe_lines).await?;
        Ok(format!("Logs for {agent} ({service}):\n{output}"))
    }

    async fn act(&self, agent: &str, action: &str) -> Result<String> {
        let service = self.service_name(agent)?;
        match action {
            "restart" => {
                self.systemd.restart(&service).await?;
            }
            "start" => {
                self.systemd.start(&service).await?;
            }
            "stop" => {
                self.systemd.stop(&service).await?;
            }
            _ => bail!("unsupported action"),
        }
        let active = self.systemd.is_active(&service).await.unwrap_or_default();
        Ok(format!("{action} requested for {service}. Current state: {active}"))
    }

    async fn run_job(&self, agent: &str, text: &str) -> Result<String> {
        self.ensure_allowed(agent)?;
        let job_id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        let job = AgentJob {
            id: job_id.clone(),
            agent: agent.to_string(),
            text: text.to_string(),
            created_at: now,
        };

        let queue_dir = self.queue_root.join(agent);
        tokio::fs::create_dir_all(&queue_dir).await?;

        let queue_file = queue_dir.join(format!("{job_id}.json"));
        let payload = serde_json::to_vec_pretty(&job)?;
        tokio::fs::write(&queue_file, payload).await?;

        let result_dir = self.results_root.join(agent);
        tokio::fs::create_dir_all(&result_dir).await?;
        let result_file = result_dir.join(format!("{job_id}.json"));

        let deadline = std::time::Instant::now() + Duration::from_secs(JOB_TIMEOUT_SECONDS);
        while std::time::Instant::now() < deadline {
            if result_file.exists() {
                let content = tokio::fs::read_to_string(&result_file).await?;
                let parsed: AgentJobResult = serde_json::from_str(&content)
                    .with_context(|| format!("invalid result payload at {}", result_file.display()))?;
                return Ok(format!(
                    "Job {} completed: {}\nSummary: {}\nResult path: {}",
                    parsed.id,
                    parsed.status,
                    parsed.summary,
                    result_file.display()
                ));
            }
            sleep(Duration::from_secs(2)).await;
        }

        Ok(format!(
            "Job {job_id} queued for {agent}, still processing. Queue path: {}",
            queue_file.display()
        ))
    }

    fn service_name(&self, agent: &str) -> Result<String> {
        self.ensure_allowed(agent)?;
        if !is_safe_service_prefix(&self.config.service_prefix) {
            bail!("invalid orchestrator service_prefix")
        }
        Ok(format!("{}{}.service", self.config.service_prefix, agent))
    }

    fn ensure_allowed(&self, agent: &str) -> Result<()> {
        if !is_safe_name(agent) {
            bail!("invalid agent name")
        }
        if !self.config.allowed_agents.iter().any(|a| a == agent) {
            bail!("agent is not allowlisted")
        }
        Ok(())
    }
}

pub fn is_safe_name(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

pub async fn run_queue_worker(
    queue_dir: &Path,
    results_dir: &Path,
    config: crate::config::Config,
) -> Result<()> {
    tokio::fs::create_dir_all(queue_dir).await?;
    tokio::fs::create_dir_all(results_dir).await?;

    loop {
        let mut entries = tokio::fs::read_dir(queue_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }

            let body = tokio::fs::read_to_string(&path).await?;
            let job: AgentJob = serde_json::from_str(&body)
                .with_context(|| format!("invalid job payload: {}", path.display()))?;

            let started = Utc::now().to_rfc3339();
            let result = crate::agent::run(
                config.clone(),
                Some(job.text.clone()),
                None,
                None,
                config.default_temperature,
                vec![],
            )
            .await;

            let (status, summary) = match result {
                Ok(()) => ("ok".to_string(), "job completed".to_string()),
                Err(e) => ("error".to_string(), format!("job failed: {e}")),
            };

            let output = AgentJobResult {
                id: job.id.clone(),
                agent: job.agent,
                status,
                summary,
                created_at: started,
                finished_at: Utc::now().to_rfc3339(),
            };

            let result_path = results_dir.join(format!("{}.json", output.id));
            tokio::fs::write(&result_path, serde_json::to_vec_pretty(&output)?).await?;
            tokio::fs::remove_file(&path).await?;
        }

        sleep(Duration::from_secs(3)).await;
    }
}

fn is_safe_service_prefix(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '@')
}
