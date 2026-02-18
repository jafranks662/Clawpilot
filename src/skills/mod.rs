use anyhow::Result;
use directories::UserDirs;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime};

pub mod scan;
pub mod skill_md;
pub mod types;

const OPEN_SKILLS_REPO_URL: &str = "https://github.com/besoeasy/open-skills";
const OPEN_SKILLS_SYNC_MARKER: &str = ".zeroclaw-open-skills-sync";
const OPEN_SKILLS_SYNC_INTERVAL_SECS: u64 = 60 * 60 * 24 * 7;

/// A skill is a user-defined or community-built capability.
/// Skills live in `~/.zeroclaw/workspace/skills/<name>/SKILL.md`
/// and can include tool definitions, prompts, and automation scripts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub version: String,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub tools: Vec<SkillTool>,
    #[serde(default)]
    pub prompts: Vec<String>,
    #[serde(skip)]
    pub location: Option<PathBuf>,
    #[serde(skip)]
    pub skill_key: String,
    #[serde(skip)]
    pub primary_env: Option<String>,
    #[serde(skip)]
    pub requires_env: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClawpilotConfig {
    #[serde(default)]
    pub skills: SkillsConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillsConfig {
    #[serde(default)]
    pub entries: HashMap<String, SkillEntryConfig>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillEntryConfig {
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default, rename = "apiKey")]
    pub api_key: Option<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub config: HashMap<String, Value>,
}

/// A tool defined by a skill (shell command, HTTP call, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillTool {
    pub name: String,
    pub description: String,
    /// "shell", "http", "script"
    pub kind: String,
    /// The command/URL/script to execute
    pub command: String,
    #[serde(default)]
    pub args: HashMap<String, String>,
}

/// Skill manifest parsed from SKILL.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SkillManifest {
    skill: SkillMeta,
    #[serde(default)]
    tools: Vec<SkillTool>,
    #[serde(default)]
    prompts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SkillMeta {
    name: String,
    description: String,
    #[serde(default = "default_version")]
    version: String,
    #[serde(default)]
    author: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    requirements: SkillRequirements,
    #[serde(default)]
    metadata: SkillMetadata,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct SkillRequirements {
    #[serde(default)]
    env: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct SkillMetadata {
    #[serde(default)]
    openclaw: OpenClawMetadata,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct OpenClawMetadata {
    #[serde(default, rename = "skillKey")]
    skill_key: Option<String>,
    #[serde(default, rename = "primaryEnv")]
    primary_env: Option<String>,
}

pub struct SkillEnvGuard {
    original: HashMap<String, Option<String>>,
}

impl Drop for SkillEnvGuard {
    fn drop(&mut self) {
        for (key, value) in &self.original {
            match value {
                Some(v) => std::env::set_var(key, v),
                None => std::env::remove_var(key),
            }
        }
    }
}

impl SkillEnvGuard {
    fn new() -> Self {
        Self {
            original: HashMap::new(),
        }
    }

    fn capture(&mut self, key: &str) {
        if !self.original.contains_key(key) {
            self.original
                .insert(key.to_string(), std::env::var(key).ok());
        }
    }
}

fn default_version() -> String {
    "0.1.0".to_string()
}

/// Load all skills from the workspace skills directory
pub fn load_skills(workspace_dir: &Path) -> Vec<Skill> {
    let mut skills = Vec::new();

    if let Some(open_skills_dir) = ensure_open_skills_repo() {
        skills.extend(load_open_skills(&open_skills_dir));
    }

    skills.extend(load_workspace_skills(workspace_dir));
    skills
}

pub fn load_skills_for_run(workspace_dir: &Path) -> Vec<Skill> {
    let skills = load_skills(workspace_dir);
    let config = load_clawpilot_config();

    skills
        .into_iter()
        .filter(|skill| is_skill_enabled(skill, &config.skills.entries))
        .collect()
}

fn apply_env_overrides_for_run_with_entries(
    skills: &[Skill],
    entries: &HashMap<String, SkillEntryConfig>,
) -> SkillEnvGuard {
    let mut guard = SkillEnvGuard::new();

    for skill in skills {
        let Some(entry) = entries.get(&skill.skill_key) else {
            continue;
        };

        for (key, value) in &entry.env {
            if std::env::var(key).is_err() {
                guard.capture(key);
                std::env::set_var(key, value);
            }
        }

        if let (Some(primary_env), Some(api_key)) = (&skill.primary_env, &entry.api_key) {
            if std::env::var(primary_env).is_err() && !entry.env.contains_key(primary_env) {
                guard.capture(primary_env);
                std::env::set_var(primary_env, api_key);
            }
        }
    }

    guard
}

pub fn apply_env_overrides_for_run(skills: &[Skill]) -> SkillEnvGuard {
    let config = load_clawpilot_config();
    apply_env_overrides_for_run_with_entries(skills, &config.skills.entries)
}

fn load_clawpilot_config() -> ClawpilotConfig {
    load_clawpilot_config_from_path(&clawpilot_config_path())
}

fn load_clawpilot_config_from_path(path: &Path) -> ClawpilotConfig {
    let Ok(content) = std::fs::read_to_string(path) else {
        return ClawpilotConfig::default();
    };

    serde_json::from_str(&content).unwrap_or_default()
}

fn clawpilot_config_path() -> PathBuf {
    if let Some(dirs) = UserDirs::new() {
        dirs.home_dir().join(".clawpilot").join("clawpilot.json")
    } else {
        PathBuf::from(".clawpilot/clawpilot.json")
    }
}

fn is_skill_enabled(skill: &Skill, entries: &HashMap<String, SkillEntryConfig>) -> bool {
    let entry = entries.get(&skill.skill_key);
    let requires_ok = skill_requirements_met(skill, entry);

    match entry.and_then(|e| e.enabled) {
        Some(false) => false,
        Some(true) => requires_ok,
        None => requires_ok,
    }
}

fn skill_requirements_met(skill: &Skill, entry: Option<&SkillEntryConfig>) -> bool {
    skill.requires_env.iter().all(|required_key| {
        std::env::var(required_key).is_ok()
            || entry.is_some_and(|e| {
                e.env.contains_key(required_key)
                    || (skill.primary_env.as_deref() == Some(required_key.as_str())
                        && e.api_key.as_ref().is_some())
            })
    })
}

fn injected_env_names(skill: &Skill, entry: Option<&SkillEntryConfig>) -> Vec<String> {
    let Some(entry) = entry else {
        return Vec::new();
    };

    let mut names: Vec<String> = entry.env.keys().cloned().collect();
    if let (Some(primary_env), Some(_)) = (&skill.primary_env, &entry.api_key) {
        if !entry.env.contains_key(primary_env) {
            names.push(primary_env.clone());
        }
    }
    names.sort();
    names.dedup();
    names
}

fn load_workspace_skills(workspace_dir: &Path) -> Vec<Skill> {
    let skills_dir = workspace_dir.join("skills");
    load_skills_from_directory(&skills_dir)
}

fn load_skills_from_directory(skills_dir: &Path) -> Vec<Skill> {
    if !skills_dir.exists() {
        return Vec::new();
    }

    let mut skills = Vec::new();

    let Ok(entries) = std::fs::read_dir(skills_dir) else {
        return skills;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        // Try SKILL.toml first, then SKILL.md
        let manifest_path = path.join("SKILL.toml");
        let md_path = path.join("SKILL.md");

        if manifest_path.exists() {
            if let Ok(skill) = load_skill_toml(&manifest_path) {
                skills.push(skill);
            }
        } else if md_path.exists() {
            if let Ok(skill) = load_skill_md(&md_path, &path) {
                skills.push(skill);
            }
        }
    }

    skills
}

fn load_open_skills(repo_dir: &Path) -> Vec<Skill> {
    let mut skills = Vec::new();

    let Ok(entries) = std::fs::read_dir(repo_dir) else {
        return skills;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let is_markdown = path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("md"));
        if !is_markdown {
            continue;
        }

        let is_readme = path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.eq_ignore_ascii_case("README.md"));
        if is_readme {
            continue;
        }

        if let Ok(skill) = load_open_skill_md(&path) {
            skills.push(skill);
        }
    }

    skills
}

fn open_skills_enabled() -> bool {
    if let Ok(raw) = std::env::var("ZEROCLAW_OPEN_SKILLS_ENABLED") {
        let value = raw.trim().to_ascii_lowercase();
        return !matches!(value.as_str(), "0" | "false" | "off" | "no");
    }

    // Keep tests deterministic and network-free by default.
    !cfg!(test)
}

fn resolve_open_skills_dir() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("ZEROCLAW_OPEN_SKILLS_DIR") {
        let trimmed = path.trim();
        if !trimmed.is_empty() {
            return Some(PathBuf::from(trimmed));
        }
    }

    UserDirs::new().map(|dirs| dirs.home_dir().join("open-skills"))
}

fn ensure_open_skills_repo() -> Option<PathBuf> {
    if !open_skills_enabled() {
        return None;
    }

    let repo_dir = resolve_open_skills_dir()?;

    if !repo_dir.exists() {
        if !clone_open_skills_repo(&repo_dir) {
            return None;
        }
        let _ = mark_open_skills_synced(&repo_dir);
        return Some(repo_dir);
    }

    if should_sync_open_skills(&repo_dir) {
        if pull_open_skills_repo(&repo_dir) {
            let _ = mark_open_skills_synced(&repo_dir);
        } else {
            tracing::warn!(
                "open-skills update failed; using local copy from {}",
                repo_dir.display()
            );
        }
    }

    Some(repo_dir)
}

fn clone_open_skills_repo(repo_dir: &Path) -> bool {
    if let Some(parent) = repo_dir.parent() {
        if let Err(err) = std::fs::create_dir_all(parent) {
            tracing::warn!(
                "failed to create open-skills parent directory {}: {err}",
                parent.display()
            );
            return false;
        }
    }

    let output = Command::new("git")
        .args(["clone", "--depth", "1", OPEN_SKILLS_REPO_URL])
        .arg(repo_dir)
        .output();

    match output {
        Ok(result) if result.status.success() => {
            tracing::info!("initialized open-skills at {}", repo_dir.display());
            true
        }
        Ok(result) => {
            let stderr = String::from_utf8_lossy(&result.stderr);
            tracing::warn!("failed to clone open-skills: {stderr}");
            false
        }
        Err(err) => {
            tracing::warn!("failed to run git clone for open-skills: {err}");
            false
        }
    }
}

fn pull_open_skills_repo(repo_dir: &Path) -> bool {
    // If user points to a non-git directory via env var, keep using it without pulling.
    if !repo_dir.join(".git").exists() {
        return true;
    }

    let output = Command::new("git")
        .arg("-C")
        .arg(repo_dir)
        .args(["pull", "--ff-only"])
        .output();

    match output {
        Ok(result) if result.status.success() => true,
        Ok(result) => {
            let stderr = String::from_utf8_lossy(&result.stderr);
            tracing::warn!("failed to pull open-skills updates: {stderr}");
            false
        }
        Err(err) => {
            tracing::warn!("failed to run git pull for open-skills: {err}");
            false
        }
    }
}

fn should_sync_open_skills(repo_dir: &Path) -> bool {
    let marker = repo_dir.join(OPEN_SKILLS_SYNC_MARKER);
    let Ok(metadata) = std::fs::metadata(marker) else {
        return true;
    };
    let Ok(modified_at) = metadata.modified() else {
        return true;
    };
    let Ok(age) = SystemTime::now().duration_since(modified_at) else {
        return true;
    };

    age >= Duration::from_secs(OPEN_SKILLS_SYNC_INTERVAL_SECS)
}

fn mark_open_skills_synced(repo_dir: &Path) -> Result<()> {
    std::fs::write(repo_dir.join(OPEN_SKILLS_SYNC_MARKER), b"synced")?;
    Ok(())
}

/// Load a skill from a SKILL.toml manifest
fn load_skill_toml(path: &Path) -> Result<Skill> {
    let content = std::fs::read_to_string(path)?;
    let manifest: SkillManifest = toml::from_str(&content)?;

    Ok(Skill {
        skill_key: manifest
            .skill
            .metadata
            .openclaw
            .skill_key
            .clone()
            .unwrap_or_else(|| manifest.skill.name.clone()),
        primary_env: manifest.skill.metadata.openclaw.primary_env.clone(),
        requires_env: manifest.skill.requirements.env.clone(),
        name: manifest.skill.name,
        description: manifest.skill.description,
        version: manifest.skill.version,
        author: manifest.skill.author,
        tags: manifest.skill.tags,
        tools: manifest.tools,
        prompts: manifest.prompts,
        location: Some(path.to_path_buf()),
    })
}

/// Load a skill from a SKILL.md file (simpler format)
fn load_skill_md(path: &Path, dir: &Path) -> Result<Skill> {
    let content = std::fs::read_to_string(path)?;
    let name = dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    Ok(Skill {
        skill_key: name.clone(),
        primary_env: None,
        requires_env: Vec::new(),
        name,
        description: extract_description(&content),
        version: "0.1.0".to_string(),
        author: None,
        tags: Vec::new(),
        tools: Vec::new(),
        prompts: vec![content],
        location: Some(path.to_path_buf()),
    })
}

fn load_open_skill_md(path: &Path) -> Result<Skill> {
    let content = std::fs::read_to_string(path)?;
    let name = path
        .file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or("open-skill")
        .to_string();

    Ok(Skill {
        skill_key: name.clone(),
        primary_env: None,
        requires_env: Vec::new(),
        name,
        description: extract_description(&content),
        version: "open-skills".to_string(),
        author: Some("besoeasy/open-skills".to_string()),
        tags: vec!["open-skills".to_string()],
        tools: Vec::new(),
        prompts: vec![content],
        location: Some(path.to_path_buf()),
    })
}

fn extract_description(content: &str) -> String {
    content
        .lines()
        .find(|line| !line.starts_with('#') && !line.trim().is_empty())
        .unwrap_or("No description")
        .trim()
        .to_string()
}

/// Build a system prompt addition from all loaded skills
pub fn skills_to_prompt(skills: &[Skill]) -> String {
    use std::fmt::Write;

    if skills.is_empty() {
        return String::new();
    }

    let mut prompt = String::from("\n## Active Skills\n\n");

    for skill in skills {
        let _ = writeln!(prompt, "### {} (v{})", skill.name, skill.version);
        let _ = writeln!(prompt, "{}", skill.description);

        if !skill.tools.is_empty() {
            prompt.push_str("Tools:\n");
            for tool in &skill.tools {
                let _ = writeln!(
                    prompt,
                    "- **{}**: {} ({})",
                    tool.name, tool.description, tool.kind
                );
            }
        }

        for p in &skill.prompts {
            prompt.push_str(p);
            prompt.push('\n');
        }

        prompt.push('\n');
    }

    prompt
}

/// Get the skills directory path
pub fn skills_dir(workspace_dir: &Path) -> PathBuf {
    workspace_dir.join("skills")
}

/// Initialize the skills directory with a README
pub fn init_skills_dir(workspace_dir: &Path) -> Result<()> {
    let dir = skills_dir(workspace_dir);
    std::fs::create_dir_all(&dir)?;

    let readme = dir.join("README.md");
    if !readme.exists() {
        std::fs::write(
            &readme,
            "# ZeroClaw Skills\n\n\
             Each subdirectory is a skill. Create a `SKILL.toml` or `SKILL.md` file inside.\n\n\
             ## SKILL.toml format\n\n\
             ```toml\n\
             [skill]\n\
             name = \"my-skill\"\n\
             description = \"What this skill does\"\n\
             version = \"0.1.0\"\n\
             author = \"your-name\"\n\
             tags = [\"productivity\", \"automation\"]\n\n\
             [[tools]]\n\
             name = \"my_tool\"\n\
             description = \"What this tool does\"\n\
             kind = \"shell\"\n\
             command = \"echo hello\"\n\
             ```\n\n\
             ## SKILL.md format (simpler)\n\n\
             Just write a markdown file with instructions for the agent.\n\
             The agent will read it and follow the instructions.\n\n\
             ## Installing community skills\n\n\
             ```bash\n\
             zeroclaw skills install <github-url>\n\
             zeroclaw skills list\n\
             ```\n",
        )?;
    }

    Ok(())
}

fn print_skills_list_table(skills: &[types::ParsedSkill]) {
    if skills.is_empty() {
        println!("No skills found in ./skills.");
        return;
    }

    println!(
        "{:<24} {:<40} {:<36} {:<8} {}",
        "NAME", "DESCRIPTION", "LOCATION", "ELIGIBLE", "REASON"
    );
    println!("{}", "-".repeat(124));

    for skill in skills {
        println!(
            "{:<24} {:<40} {:<36} {:<8} {}",
            skill.frontmatter.name,
            skill.frontmatter.description,
            skill.skill_dir.display(),
            skill.eligible,
            skill.reason
        );
    }
}

fn print_skill_detail(skill: &types::ParsedSkill) {
    println!("name: {}", skill.frontmatter.name);
    println!("description: {}", skill.frontmatter.description);
    println!(
        "metadata: {}",
        skill
            .frontmatter
            .metadata
            .as_ref()
            .map(std::string::ToString::to_string)
            .unwrap_or_else(|| "null".to_string())
    );
    println!("path: {}", skill.skill_md_path.display());
    println!("location: {}", skill.skill_dir.display());
    println!("eligible: {}", skill.eligible);
    println!("reason: {}", skill.reason);
}

/// Recursively copy a directory (used as fallback when symlinks aren't available)
#[cfg(any(windows, not(unix)))]
fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<()> {
    std::fs::create_dir_all(dest)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dest_path)?;
        } else {
            std::fs::copy(&src_path, &dest_path)?;
        }
    }
    Ok(())
}

/// Handle the `skills` CLI command
#[allow(clippy::too_many_lines)]
pub fn handle_command(command: crate::SkillCommands, workspace_dir: &Path) -> Result<()> {
    match command {
        crate::SkillCommands::List => {
            let skills = scan::scan_skills(None)?;
            print_skills_list_table(&skills);
            Ok(())
        }
        crate::SkillCommands::Show { name } => {
            let skills = scan::scan_skills(None)?;
            let Some(skill) = skills.into_iter().find(|s| s.frontmatter.name == name) else {
                anyhow::bail!("Skill not found: {name}");
            };

            print_skill_detail(&skill);
            Ok(())
        }
        crate::SkillCommands::Install { source } => {
            println!("Installing skill from: {source}");

            let skills_path = skills_dir(workspace_dir);
            std::fs::create_dir_all(&skills_path)?;

            if source.starts_with("https://") || source.starts_with("http://") {
                // Git clone
                let output = std::process::Command::new("git")
                    .args(["clone", "--depth", "1", &source])
                    .current_dir(&skills_path)
                    .output()?;

                if output.status.success() {
                    println!(
                        "  {} Skill installed successfully!",
                        console::style("✓").green().bold()
                    );
                    println!("  Restart `zeroclaw channel start` to activate.");
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    anyhow::bail!("Git clone failed: {stderr}");
                }
            } else {
                // Local path — symlink or copy
                let src = PathBuf::from(&source);
                if !src.exists() {
                    anyhow::bail!("Source path does not exist: {source}");
                }
                let name = src.file_name().unwrap_or_default();
                let dest = skills_path.join(name);

                #[cfg(unix)]
                {
                    std::os::unix::fs::symlink(&src, &dest)?;
                    println!(
                        "  {} Skill linked: {}",
                        console::style("✓").green().bold(),
                        dest.display()
                    );
                }
                #[cfg(windows)]
                {
                    // On Windows, try symlink first (requires admin or developer mode),
                    // fall back to directory junction, then copy
                    use std::os::windows::fs::symlink_dir;
                    if symlink_dir(&src, &dest).is_ok() {
                        println!(
                            "  {} Skill linked: {}",
                            console::style("✓").green().bold(),
                            dest.display()
                        );
                    } else {
                        // Try junction as fallback (works without admin)
                        let junction_result = std::process::Command::new("cmd")
                            .args(["/C", "mklink", "/J"])
                            .arg(&dest)
                            .arg(&src)
                            .output();

                        if junction_result.is_ok() && junction_result.unwrap().status.success() {
                            println!(
                                "  {} Skill linked (junction): {}",
                                console::style("✓").green().bold(),
                                dest.display()
                            );
                        } else {
                            // Final fallback: copy the directory
                            copy_dir_recursive(&src, &dest)?;
                            println!(
                                "  {} Skill copied: {}",
                                console::style("✓").green().bold(),
                                dest.display()
                            );
                        }
                    }
                }
                #[cfg(not(any(unix, windows)))]
                {
                    // On other platforms, copy the directory
                    copy_dir_recursive(&src, &dest)?;
                    println!(
                        "  {} Skill copied: {}",
                        console::style("✓").green().bold(),
                        dest.display()
                    );
                }
            }

            Ok(())
        }
        crate::SkillCommands::Remove { name } => {
            // Reject path traversal attempts
            if name.contains("..") || name.contains('/') || name.contains('\\') {
                anyhow::bail!("Invalid skill name: {name}");
            }

            let skill_path = skills_dir(workspace_dir).join(&name);

            // Verify the resolved path is actually inside the skills directory
            let canonical_skills = skills_dir(workspace_dir)
                .canonicalize()
                .unwrap_or_else(|_| skills_dir(workspace_dir));
            if let Ok(canonical_skill) = skill_path.canonicalize() {
                if !canonical_skill.starts_with(&canonical_skills) {
                    anyhow::bail!("Skill path escapes skills directory: {name}");
                }
            }

            if !skill_path.exists() {
                anyhow::bail!("Skill not found: {name}");
            }

            std::fs::remove_dir_all(&skill_path)?;
            println!(
                "  {} Skill '{}' removed.",
                console::style("✓").green().bold(),
                name
            );
            Ok(())
        }
    }
}

#[cfg(test)]
#[allow(clippy::similar_names)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn load_empty_skills_dir() {
        let dir = tempfile::tempdir().unwrap();
        let skills = load_skills(dir.path());
        assert!(skills.is_empty());
    }

    #[test]
    fn load_skill_from_toml() {
        let dir = tempfile::tempdir().unwrap();
        let skills_dir = dir.path().join("skills");
        let skill_dir = skills_dir.join("test-skill");
        fs::create_dir_all(&skill_dir).unwrap();

        fs::write(
            skill_dir.join("SKILL.toml"),
            r#"
[skill]
name = "test-skill"
description = "A test skill"
version = "1.0.0"
tags = ["test"]

[[tools]]
name = "hello"
description = "Says hello"
kind = "shell"
command = "echo hello"
"#,
        )
        .unwrap();

        let skills = load_skills(dir.path());
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "test-skill");
        assert_eq!(skills[0].tools.len(), 1);
        assert_eq!(skills[0].tools[0].name, "hello");
    }

    #[test]
    fn load_skill_from_md() {
        let dir = tempfile::tempdir().unwrap();
        let skills_dir = dir.path().join("skills");
        let skill_dir = skills_dir.join("md-skill");
        fs::create_dir_all(&skill_dir).unwrap();

        fs::write(
            skill_dir.join("SKILL.md"),
            "# My Skill\nThis skill does cool things.\n",
        )
        .unwrap();

        let skills = load_skills(dir.path());
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "md-skill");
        assert!(skills[0].description.contains("cool things"));
    }

    #[test]
    fn skills_to_prompt_empty() {
        let prompt = skills_to_prompt(&[]);
        assert!(prompt.is_empty());
    }

    #[test]
    fn skills_to_prompt_with_skills() {
        let skills = vec![Skill {
            name: "test".to_string(),
            description: "A test".to_string(),
            version: "1.0.0".to_string(),
            author: None,
            tags: vec![],
            tools: vec![],
            prompts: vec!["Do the thing.".to_string()],
            location: None,
            skill_key: "test".to_string(),
            primary_env: None,
            requires_env: vec![],
        }];
        let prompt = skills_to_prompt(&skills);
        assert!(prompt.contains("test"));
        assert!(prompt.contains("Do the thing"));
    }

    #[test]
    fn init_skills_creates_readme() {
        let dir = tempfile::tempdir().unwrap();
        init_skills_dir(dir.path()).unwrap();
        assert!(dir.path().join("skills").join("README.md").exists());
    }

    #[test]
    fn init_skills_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        init_skills_dir(dir.path()).unwrap();
        init_skills_dir(dir.path()).unwrap(); // second call should not fail
        assert!(dir.path().join("skills").join("README.md").exists());
    }

    #[test]
    fn load_nonexistent_dir() {
        let dir = tempfile::tempdir().unwrap();
        let fake = dir.path().join("nonexistent");
        let skills = load_skills(&fake);
        assert!(skills.is_empty());
    }

    #[test]
    fn load_ignores_files_in_skills_dir() {
        let dir = tempfile::tempdir().unwrap();
        let skills_dir = dir.path().join("skills");
        fs::create_dir_all(&skills_dir).unwrap();
        // A file, not a directory — should be ignored
        fs::write(skills_dir.join("not-a-skill.txt"), "hello").unwrap();
        let skills = load_skills(dir.path());
        assert!(skills.is_empty());
    }

    #[test]
    fn load_ignores_dir_without_manifest() {
        let dir = tempfile::tempdir().unwrap();
        let skills_dir = dir.path().join("skills");
        let empty_skill = skills_dir.join("empty-skill");
        fs::create_dir_all(&empty_skill).unwrap();
        // Directory exists but no SKILL.toml or SKILL.md
        let skills = load_skills(dir.path());
        assert!(skills.is_empty());
    }

    #[test]
    fn load_multiple_skills() {
        let dir = tempfile::tempdir().unwrap();
        let skills_dir = dir.path().join("skills");

        for name in ["alpha", "beta", "gamma"] {
            let skill_dir = skills_dir.join(name);
            fs::create_dir_all(&skill_dir).unwrap();
            fs::write(
                skill_dir.join("SKILL.md"),
                format!("# {name}\nSkill {name} description.\n"),
            )
            .unwrap();
        }

        let skills = load_skills(dir.path());
        assert_eq!(skills.len(), 3);
    }

    #[test]
    fn toml_skill_with_multiple_tools() {
        let dir = tempfile::tempdir().unwrap();
        let skills_dir = dir.path().join("skills");
        let skill_dir = skills_dir.join("multi-tool");
        fs::create_dir_all(&skill_dir).unwrap();

        fs::write(
            skill_dir.join("SKILL.toml"),
            r#"
[skill]
name = "multi-tool"
description = "Has many tools"
version = "2.0.0"
author = "tester"
tags = ["automation", "devops"]

[[tools]]
name = "build"
description = "Build the project"
kind = "shell"
command = "cargo build"

[[tools]]
name = "test"
description = "Run tests"
kind = "shell"
command = "cargo test"

[[tools]]
name = "deploy"
description = "Deploy via HTTP"
kind = "http"
command = "https://api.example.com/deploy"
"#,
        )
        .unwrap();

        let skills = load_skills(dir.path());
        assert_eq!(skills.len(), 1);
        let s = &skills[0];
        assert_eq!(s.name, "multi-tool");
        assert_eq!(s.version, "2.0.0");
        assert_eq!(s.author.as_deref(), Some("tester"));
        assert_eq!(s.tags, vec!["automation", "devops"]);
        assert_eq!(s.tools.len(), 3);
        assert_eq!(s.tools[0].name, "build");
        assert_eq!(s.tools[1].kind, "shell");
        assert_eq!(s.tools[2].kind, "http");
    }

    #[test]
    fn toml_skill_minimal() {
        let dir = tempfile::tempdir().unwrap();
        let skills_dir = dir.path().join("skills");
        let skill_dir = skills_dir.join("minimal");
        fs::create_dir_all(&skill_dir).unwrap();

        fs::write(
            skill_dir.join("SKILL.toml"),
            r#"
[skill]
name = "minimal"
description = "Bare minimum"
"#,
        )
        .unwrap();

        let skills = load_skills(dir.path());
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].version, "0.1.0"); // default version
        assert!(skills[0].author.is_none());
        assert!(skills[0].tags.is_empty());
        assert!(skills[0].tools.is_empty());
    }

    #[test]
    fn toml_skill_invalid_syntax_skipped() {
        let dir = tempfile::tempdir().unwrap();
        let skills_dir = dir.path().join("skills");
        let skill_dir = skills_dir.join("broken");
        fs::create_dir_all(&skill_dir).unwrap();

        fs::write(skill_dir.join("SKILL.toml"), "this is not valid toml {{{{").unwrap();

        let skills = load_skills(dir.path());
        assert!(skills.is_empty()); // broken skill is skipped
    }

    #[test]
    fn md_skill_heading_only() {
        let dir = tempfile::tempdir().unwrap();
        let skills_dir = dir.path().join("skills");
        let skill_dir = skills_dir.join("heading-only");
        fs::create_dir_all(&skill_dir).unwrap();

        fs::write(skill_dir.join("SKILL.md"), "# Just a Heading\n").unwrap();

        let skills = load_skills(dir.path());
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].description, "No description");
    }

    #[test]
    fn skills_to_prompt_includes_tools() {
        let skills = vec![Skill {
            name: "weather".to_string(),
            description: "Get weather".to_string(),
            version: "1.0.0".to_string(),
            author: None,
            tags: vec![],
            tools: vec![SkillTool {
                name: "get_weather".to_string(),
                description: "Fetch forecast".to_string(),
                kind: "shell".to_string(),
                command: "curl wttr.in".to_string(),
                args: HashMap::new(),
            }],
            prompts: vec![],
            location: None,
            skill_key: "weather".to_string(),
            primary_env: None,
            requires_env: vec![],
        }];
        let prompt = skills_to_prompt(&skills);
        assert!(prompt.contains("weather"));
        assert!(prompt.contains("get_weather"));
        assert!(prompt.contains("Fetch forecast"));
        assert!(prompt.contains("shell"));
    }

    #[test]
    fn skills_dir_path() {
        let base = std::path::Path::new("/home/user/.zeroclaw");
        let dir = skills_dir(base);
        assert_eq!(dir, PathBuf::from("/home/user/.zeroclaw/skills"));
    }

    #[test]
    fn toml_prefers_over_md() {
        let dir = tempfile::tempdir().unwrap();
        let skills_dir = dir.path().join("skills");
        let skill_dir = skills_dir.join("dual");
        fs::create_dir_all(&skill_dir).unwrap();

        fs::write(
            skill_dir.join("SKILL.toml"),
            "[skill]\nname = \"from-toml\"\ndescription = \"TOML wins\"\n",
        )
        .unwrap();
        fs::write(skill_dir.join("SKILL.md"), "# From MD\nMD description\n").unwrap();

        let skills = load_skills(dir.path());
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "from-toml"); // TOML takes priority
    }

    #[test]
    fn skill_env_injection_and_restore() {
        let dynamic = format!("ZEROCLAW_TEST_ENV_{}", std::process::id());
        std::env::remove_var(&dynamic);

        let skill = Skill {
            name: "env-skill".into(),
            description: "d".into(),
            version: "0.1.0".into(),
            author: None,
            tags: vec![],
            tools: vec![],
            prompts: vec![],
            location: None,
            skill_key: "env-key".into(),
            primary_env: Some("ZEROCLAW_PRIMARY_ENV_TEST".into()),
            requires_env: vec![],
        };

        std::env::set_var("ZEROCLAW_PRIMARY_ENV_TEST", "already-set");

        let mut entries = HashMap::new();
        entries.insert(
            "env-key".into(),
            SkillEntryConfig {
                enabled: Some(true),
                api_key: Some("secret-key".into()),
                env: HashMap::from([(dynamic.clone(), "from-config".into())]),
                config: HashMap::new(),
            },
        );

        {
            let _guard = apply_env_overrides_for_run_with_entries(&[skill], &entries);
            assert_eq!(std::env::var(&dynamic).as_deref(), Ok("from-config"));
            assert_eq!(
                std::env::var("ZEROCLAW_PRIMARY_ENV_TEST").as_deref(),
                Ok("already-set")
            );
        }

        assert!(std::env::var(&dynamic).is_err());
        assert_eq!(
            std::env::var("ZEROCLAW_PRIMARY_ENV_TEST").as_deref(),
            Ok("already-set")
        );
        std::env::remove_var("ZEROCLAW_PRIMARY_ENV_TEST");
    }

    #[test]
    fn skill_api_key_maps_to_primary_env() {
        std::env::remove_var("ZEROCLAW_PRIMARY_ENV_API_KEY_TEST");

        let skill = Skill {
            name: "api-skill".into(),
            description: "d".into(),
            version: "0.1.0".into(),
            author: None,
            tags: vec![],
            tools: vec![],
            prompts: vec![],
            location: None,
            skill_key: "api-key".into(),
            primary_env: Some("ZEROCLAW_PRIMARY_ENV_API_KEY_TEST".into()),
            requires_env: vec!["ZEROCLAW_PRIMARY_ENV_API_KEY_TEST".into()],
        };

        let mut entries = HashMap::new();
        entries.insert(
            "api-key".into(),
            SkillEntryConfig {
                enabled: Some(true),
                api_key: Some("mapped-secret".into()),
                env: HashMap::new(),
                config: HashMap::new(),
            },
        );

        assert!(skill_requirements_met(&skill, entries.get("api-key")));

        {
            let _guard = apply_env_overrides_for_run_with_entries(&[skill], &entries);
            assert_eq!(
                std::env::var("ZEROCLAW_PRIMARY_ENV_API_KEY_TEST").as_deref(),
                Ok("mapped-secret")
            );
        }

        assert!(std::env::var("ZEROCLAW_PRIMARY_ENV_API_KEY_TEST").is_err());
    }

}

#[cfg(test)]
mod symlink_tests;
