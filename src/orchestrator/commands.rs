use anyhow::{bail, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrchestratorCommand {
    Help,
    Status,
    Logs { agent: String, lines: Option<usize> },
    Restart { agent: String },
    Start { agent: String },
    Stop { agent: String },
    Run { agent: String, text: String },
}

pub fn parse_command(input: &str) -> Result<OrchestratorCommand> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        bail!("empty command")
    }

    let mut parts = trimmed.split_whitespace();
    let Some(command) = parts.next() else {
        bail!("missing command")
    };

    match command {
        "/help" => Ok(OrchestratorCommand::Help),
        "/status" => Ok(OrchestratorCommand::Status),
        "/logs" => {
            let Some(agent) = parts.next() else {
                bail!("usage: /logs <agent> [N]")
            };
            let lines = parts
                .next()
                .map(|value| value.parse::<usize>())
                .transpose()
                .map_err(|_| anyhow::anyhow!("invalid line count"))?;
            Ok(OrchestratorCommand::Logs {
                agent: agent.to_string(),
                lines,
            })
        }
        "/restart" => {
            let Some(agent) = parts.next() else {
                bail!("usage: /restart <agent>")
            };
            Ok(OrchestratorCommand::Restart {
                agent: agent.to_string(),
            })
        }
        "/start" => {
            let Some(agent) = parts.next() else {
                bail!("usage: /start <agent>")
            };
            Ok(OrchestratorCommand::Start {
                agent: agent.to_string(),
            })
        }
        "/stop" => {
            let Some(agent) = parts.next() else {
                bail!("usage: /stop <agent>")
            };
            Ok(OrchestratorCommand::Stop {
                agent: agent.to_string(),
            })
        }
        "/run" => {
            let Some(agent) = parts.next() else {
                bail!("usage: /run <agent> <text...>")
            };
            let text: String = parts.collect::<Vec<_>>().join(" ");
            if text.trim().is_empty() {
                bail!("usage: /run <agent> <text...>")
            }
            Ok(OrchestratorCommand::Run {
                agent: agent.to_string(),
                text,
            })
        }
        _ => bail!("unknown command: {command}"),
    }
}
