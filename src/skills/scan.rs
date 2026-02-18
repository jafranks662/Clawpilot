use crate::skills::skill_md::parse_skill_md;
use crate::skills::types::ParsedSkill;
use anyhow::Result;
use std::path::{Path, PathBuf};

const SKILL_MARKDOWN_FILE: &str = "SKILL.md";

pub fn scan_skills(root: Option<&Path>) -> Result<Vec<ParsedSkill>> {
    let root = root.unwrap_or_else(|| Path::new("./skills"));

    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut found = Vec::new();
    let mut stack = vec![root.to_path_buf()];

    while let Some(current) = stack.pop() {
        let entries = match std::fs::read_dir(&current) {
            Ok(entries) => entries,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                stack.push(path);
                continue;
            }

            if path
                .file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value == SKILL_MARKDOWN_FILE)
            {
                found.push(path);
            }
        }
    }

    found.sort();

    let mut parsed_skills = Vec::new();
    for skill_md_path in found {
        let frontmatter = parse_skill_md(&skill_md_path)?;
        let skill_dir = skill_md_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(PathBuf::new);

        parsed_skills.push(ParsedSkill {
            frontmatter,
            skill_dir,
            skill_md_path,
            eligible: true,
            reason: String::new(),
        });
    }

    Ok(parsed_skills)
}
