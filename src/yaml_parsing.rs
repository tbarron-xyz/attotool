use serde::Deserialize;
use serde_yaml::{Mapping, Value as YamlValue};
use std::env;
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
pub struct Config {
    model: Option<String>,
}

pub static DEFAULT_SYSTEM_PROMPT_YAML: &str =
    include_str!("../system_prompt.yaml");

pub fn get_default_model() -> String {
    let config_path = format!(
        "{}/.config/attotool/config.yaml",
        env::var("HOME").expect("HOME not set")
    );
    let mut model = "mistralai/mistral-small-3.1-24b-instruct".to_string();
    if Path::new(&config_path).exists() {
        if let Ok(content) = fs::read_to_string(&config_path) {
            if let Ok(config) = serde_yaml::from_str::<Config>(&content) {
                if let Some(m) = config.model {
                    model = m;
                }
            }
        }
    }
    model
}

pub fn merge_yaml(
    base: &serde_yaml::Value,
    user: &serde_yaml::Value,
) -> serde_yaml::Value {
    let mut result = base.clone();
    if let serde_yaml::Value::Mapping(ref mut res_map) = result {
        if let serde_yaml::Value::Mapping(user_map) = user {
            for (key, user_val) in user_map {
                res_map.insert(key.clone(), user_val.clone());
            }
        }
    }
    result
}

pub fn format_system_prompt_from_yaml(
    yaml: &serde_yaml::Value,
    current_dir: &Path,
    disable_agents_md: bool,
    plan_mode: bool,
    available_tools_text: &str,
) -> String {
    let current_dir_formatted = yaml["current_dir"]
        .as_str()
        .unwrap_or("")
        .replace("{}", &current_dir.display().to_string());
    let agents_md_formatted =
        if !disable_agents_md && fs::metadata("AGENTS.md").is_ok() {
            yaml["agents_md"].as_str().unwrap_or("")
        } else {
            ""
        };
    let plan_formatted = if plan_mode {
        yaml["plan_mode"].as_str().unwrap_or("")
    } else {
        ""
    };
    let system_content = format!(
        "{}\n\n{}\n\n{}{}{}\n\n{}\n\n{}",
        yaml["role_and_format"].as_str().unwrap_or(""),
        yaml["task"].as_str().unwrap_or(""),
        current_dir_formatted,
        agents_md_formatted,
        plan_formatted,
        yaml["tools"]
            .as_str()
            .unwrap_or("")
            .replace("{}", available_tools_text),
        yaml["examples"].as_str().unwrap_or("").trim_start_matches('\n')
    );
    system_content
}

pub fn format_system_prompt(
    current_dir: &Path,
    disable_agents_md: bool,
    plan_mode: bool,
    available_tools_text: &str,
) -> String {
    let base_yaml: serde_yaml::Value =
        serde_yaml::from_str(DEFAULT_SYSTEM_PROMPT_YAML)
            .expect("Failed to parse default system prompt YAML");
    let home = env::var("HOME").unwrap_or("".to_string());
    let user_config_path =
        format!("{}/.config/attotool/system_prompt.yaml", home);
    let user_yaml = if Path::new(&user_config_path).exists() {
        Some(
            fs::read_to_string(&user_config_path)
                .expect("Failed to read user config"),
        )
    } else {
        None
    };
    let merged_yaml = if let Some(ref user_str) = user_yaml {
        let user_val: serde_yaml::Value = serde_yaml::from_str(user_str)
            .expect("Failed to parse user system prompt YAML");
        merge_yaml(&base_yaml, &user_val)
    } else {
        base_yaml
    };
    format_system_prompt_from_yaml(
        &merged_yaml,
        current_dir,
        disable_agents_md,
        plan_mode,
        available_tools_text,
    )
}

pub fn parse_and_normalize_yaml(
    input: &str,
    verbose: bool,
) -> Result<Mapping, Box<dyn std::error::Error>> {
    if let Ok(value) = serde_yaml::from_str::<YamlValue>(input) {
        if let YamlValue::Mapping(mapping) = value {
            if mapping.len() > 1 {
                if verbose {
                    println!(
                        "Removed {} additional tool(s) from multi-tool response",
                        mapping.len() - 1
                    );
                }
                let mut new_mapping = Mapping::new();
                if let Some((key, val)) = mapping.iter().next() {
                    new_mapping.insert(key.clone(), val.clone());
                }
                return Ok(new_mapping);
            } else {
                return Ok(mapping);
            }
        }
    }
    Err("Invalid YAML".into())
}

pub fn parse_tool_response_yaml(
    trimmed: &str,
    verbose: bool,
) -> Result<Mapping, Box<dyn std::error::Error>> {
    // First, try parsing the entire trimmed response as YAML
    if let Ok(normalized) = parse_and_normalize_yaml(trimmed, verbose) {
        return Ok(normalized);
    }
    // If parsing the whole failed, try splitting by \n\n and parse the first part
    let yaml_candidate = if let Some(pos) = trimmed.find("\n\n") {
        &trimmed[..pos]
    } else {
        trimmed
    };
    parse_and_normalize_yaml(yaml_candidate, verbose)
}
