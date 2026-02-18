use crate::skills::types::SkillFrontmatter;
use anyhow::{bail, Context, Result};
use std::path::Path;

pub fn parse_skill_md(path: &Path) -> Result<SkillFrontmatter> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read skill markdown at {}", path.display()))?;
    parse_skill_md_content(&content)
}

pub fn parse_skill_md_content(content: &str) -> Result<SkillFrontmatter> {
    let mut lines = content.lines();
    let Some(first_line) = lines.next() else {
        bail!("missing frontmatter start delimiter");
    };

    if first_line.trim() != "---" {
        bail!("missing frontmatter start delimiter");
    }

    let mut frontmatter_lines = Vec::new();
    let mut found_end = false;

    for line in lines {
        if line.trim() == "---" {
            found_end = true;
            break;
        }
        frontmatter_lines.push(line);
    }

    if !found_end {
        bail!("missing frontmatter end delimiter");
    }

    let mut name: Option<String> = None;
    let mut description: Option<String> = None;
    let mut metadata: Option<serde_json::Value> = None;

    for line in frontmatter_lines {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let Some((raw_key, raw_value)) = trimmed.split_once(':') else {
            continue;
        };

        let key = raw_key.trim();
        let value = raw_value.trim();

        match key {
            "name" => name = Some(value.to_string()),
            "description" => description = Some(value.to_string()),
            "metadata" => {
                let parsed: serde_json::Value = serde_json::from_str(value)
                    .context("metadata must be valid JSON object")?;
                if !parsed.is_object() {
                    bail!("metadata must be a JSON object");
                }
                metadata = Some(parsed);
            }
            _ => {}
        }
    }

    let name = name
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow::anyhow!("missing required frontmatter key: name"))?;
    let description = description
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow::anyhow!("missing required frontmatter key: description"))?;

    Ok(SkillFrontmatter {
        name,
        description,
        metadata,
    })
}

#[cfg(test)]
mod tests {
    use super::parse_skill_md_content;

    #[test]
    fn parse_valid_skill_md() {
        let parsed = parse_skill_md_content(
            r#"---
name: test_skill
description: A test skill
metadata: {"category":"build","priority":1}
extra: ignored
---
# Body
"#,
        )
        .expect("frontmatter should parse");

        assert_eq!(parsed.name, "test_skill");
        assert_eq!(parsed.description, "A test skill");
        assert_eq!(parsed.metadata.unwrap()["category"], "build");
    }

    #[test]
    fn parse_invalid_or_missing_frontmatter() {
        let missing = parse_skill_md_content("name: nope\n");
        assert!(missing.is_err());

        let unclosed = parse_skill_md_content("---\nname: a\ndescription: b\n");
        assert!(unclosed.is_err());
    }

    #[test]
    fn metadata_must_be_valid_json() {
        let invalid = parse_skill_md_content(
            "---\nname: x\ndescription: y\nmetadata: {invalid json}\n---\n",
        );
        assert!(invalid.is_err());

        let not_object = parse_skill_md_content(
            "---\nname: x\ndescription: y\nmetadata: [1,2,3]\n---\n",
        );
        assert!(not_object.is_err());
    }
}
