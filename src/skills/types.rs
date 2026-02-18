use serde_json::Value;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub struct SkillFrontmatter {
    pub name: String,
    pub description: String,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedSkill {
    pub frontmatter: SkillFrontmatter,
    pub skill_dir: PathBuf,
    pub skill_md_path: PathBuf,
    pub eligible: bool,
    pub reason: String,
}
