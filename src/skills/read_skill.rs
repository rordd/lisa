//! `read_skill` tool — lets the LLM load a skill's full content on demand in Compact mode.

use crate::skills::Skill;
use crate::tools::traits::{Tool, ToolResult};
use async_trait::async_trait;
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;

/// Tool that reads a skill's full SKILL.md content by name.
/// Only registered when `prompt_injection_mode = Compact`.
pub struct ReadSkillTool {
    /// Map from skill name to its file location.
    skill_locations: HashMap<String, PathBuf>,
}

impl ReadSkillTool {
    /// Build from loaded skills list. Only skills with a known `location` are readable.
    pub fn from_skills(skills: &[Skill]) -> Self {
        let skill_locations = skills
            .iter()
            .filter_map(|s| {
                s.location
                    .as_ref()
                    .map(|loc| (s.name.clone(), loc.clone()))
            })
            .collect();
        Self { skill_locations }
    }
}

#[async_trait]
impl Tool for ReadSkillTool {
    fn name(&self) -> &str {
        "read_skill"
    }

    fn description(&self) -> &str {
        "Read the full instructions of a skill by name. Use this to load skill details on demand in compact mode."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "The skill name to read (must match one of the available skills)"
                }
            },
            "required": ["name"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let name = args
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required 'name' parameter"))?;

        let path = match self.skill_locations.get(name) {
            Some(p) => p,
            None => {
                let available: Vec<&str> = self.skill_locations.keys().map(|s| s.as_str()).collect();
                return Ok(ToolResult {
                    success: false,
                    output: format!(
                        "Unknown skill '{}'. Available skills: {}",
                        name,
                        available.join(", ")
                    ),
                    error: None,
                });
            }
        };

        match tokio::fs::read_to_string(path).await {
            Ok(content) => Ok(ToolResult {
                success: true,
                output: content,
                error: None,
            }),
            Err(e) => Ok(ToolResult {
                success: false,
                output: format!("Failed to read skill file: {e}"),
                error: Some(e.to_string()),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn make_skill(name: &str, location: Option<PathBuf>) -> Skill {
        Skill {
            name: name.to_string(),
            description: format!("Test skill {name}"),
            version: "0.1.0".to_string(),
            author: None,
            tags: Vec::new(),
            tools: Vec::new(),
            prompts: Vec::new(),
            location,
            always: false,
            channels: Vec::new(),
        }
    }

    #[test]
    fn from_skills_builds_location_map() {
        let skills = vec![
            make_skill("alpha", Some(PathBuf::from("/skills/alpha/SKILL.md"))),
            make_skill("beta", None), // no location — should be excluded
            make_skill("gamma", Some(PathBuf::from("/skills/gamma/SKILL.md"))),
        ];
        let tool = ReadSkillTool::from_skills(&skills);
        assert_eq!(tool.skill_locations.len(), 2);
        assert!(tool.skill_locations.contains_key("alpha"));
        assert!(tool.skill_locations.contains_key("gamma"));
        assert!(!tool.skill_locations.contains_key("beta"));
    }

    #[tokio::test]
    async fn execute_returns_error_for_unknown_skill() {
        let tool = ReadSkillTool::from_skills(&[make_skill(
            "known",
            Some(PathBuf::from("/tmp/fake.md")),
        )]);
        let result = tool
            .execute(json!({"name": "unknown"}))
            .await
            .unwrap();
        assert!(!result.success);
        assert!(result.output.contains("Unknown skill 'unknown'"));
        assert!(result.output.contains("known"));
    }

    #[tokio::test]
    async fn execute_reads_skill_file() {
        let mut tmp = NamedTempFile::new().unwrap();
        writeln!(tmp, "# My Skill\nDo amazing things.").unwrap();
        let path = tmp.path().to_path_buf();

        let tool = ReadSkillTool::from_skills(&[make_skill("my-skill", Some(path))]);
        let result = tool.execute(json!({"name": "my-skill"})).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("My Skill"));
        assert!(result.output.contains("Do amazing things."));
    }

    #[tokio::test]
    async fn execute_missing_name_param() {
        let tool = ReadSkillTool::from_skills(&[]);
        let result = tool.execute(json!({})).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn execute_handles_missing_file() {
        let tool = ReadSkillTool::from_skills(&[make_skill(
            "gone",
            Some(PathBuf::from("/nonexistent/path/SKILL.md")),
        )]);
        let result = tool.execute(json!({"name": "gone"})).await.unwrap();
        assert!(!result.success);
        assert!(result.output.contains("Failed to read skill file"));
    }

    #[test]
    fn tool_metadata() {
        let tool = ReadSkillTool::from_skills(&[]);
        assert_eq!(tool.name(), "read_skill");
        assert!(!tool.description().is_empty());
        let schema = tool.parameters_schema();
        assert_eq!(schema["type"], "object");
        assert_eq!(schema["properties"]["name"]["type"], "string");
        assert_eq!(schema["required"], json!(["name"]));
    }
}
