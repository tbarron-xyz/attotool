use async_openai::{
    Client,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
        ChatCompletionRequestSystemMessageContent, CreateChatCompletionRequest,
    },
};
use serde_yaml::{Mapping, Value as YamlValue};
use std::env;

use crate::yaml_utilities::parse_tool_response_yaml;

#[derive(Debug)]
pub enum PromptType {
    Approval(String),
    Clarification(String),
}

#[derive(Debug)]
pub enum LoopError {
    Prompt(PromptType),
    Other(Box<dyn std::error::Error + Send + Sync>),
}

impl std::fmt::Display for LoopError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            LoopError::Prompt(PromptType::Approval(msg)) => {
                write!(f, "Approval: {}", msg)
            }
            LoopError::Prompt(PromptType::Clarification(msg)) => {
                write!(f, "Clarification: {}", msg)
            }
            LoopError::Other(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for LoopError {}

impl From<String> for LoopError {
    fn from(s: String) -> Self {
        LoopError::Other(s.into())
    }
}

impl From<serde_yaml::Error> for LoopError {
    fn from(e: serde_yaml::Error) -> Self {
        LoopError::Other(Box::new(e))
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for LoopError {
    fn from(e: Box<dyn std::error::Error + Send + Sync>) -> Self {
        LoopError::Other(e)
    }
}

pub async fn choose_tool(
    history: Vec<ChatCompletionRequestMessage>,
    model: &str,
    retries: u32,
    max_tokens: u32,
    base_url: &str,
    verbose: bool,
    yolo: bool,
    disable_agents_md: bool,
    plan_mode: bool,
) -> Result<Mapping, Box<dyn std::error::Error + Send + Sync>> {
    let api_key =
        env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY must be set");
    let client = Client::with_config(
        async_openai::config::OpenAIConfig::new()
            .with_api_base(base_url)
            .with_api_key(api_key),
    );

    let tools = crate::tools::get_tools(yolo, plan_mode);
    let available_tools_text =
        tools.iter().map(|t| t.format()).collect::<Vec<_>>().join("\n");
    let current_dir = std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("unknown"));
    let system_content = crate::yaml_utilities::format_system_prompt(
        &current_dir,
        disable_agents_md,
        plan_mode,
        &available_tools_text,
        yolo,
    );
    let system_message = ChatCompletionRequestMessage::System(
        ChatCompletionRequestSystemMessage {
            content: ChatCompletionRequestSystemMessageContent::Text(
                system_content,
            ),
            name: None,
        },
    );

    let mut messages = vec![system_message];
    messages.extend(history);

    let request = CreateChatCompletionRequest {
        model: model.to_string(),
        messages,
        max_completion_tokens: Some(max_tokens),
        ..Default::default()
    };

    for attempt in 0..retries {
        if verbose {
            println!("Sending openai request");
        }
        let response = client.chat().create(request.clone()).await?;
        let choice = response.choices.first().ok_or("No response")?;
        let content = choice.message.content.as_ref().ok_or("No content")?;
        let trimmed = content.trim();
        if verbose {
            println!(
                "API Response Content (choose_tool, attempt {}):\n{}",
                attempt + 1,
                content
            );
        }
        if !trimmed.is_empty() {
            if let Ok(normalized) = parse_tool_response_yaml(trimmed, verbose) {
                return Ok(normalized);
            }
            // Fallback: return a mapping for finish tool
            let finish_tool_name = if plan_mode {
                "finish_planning"
            } else {
                "finish_task"
            };
            let mut args = Mapping::new();
            args.insert(
                YamlValue::String("message".to_string()),
                YamlValue::String(trimmed.to_string()),
            );
            let mut mapping = Mapping::new();
            mapping.insert(
                YamlValue::String(finish_tool_name.to_string()),
                YamlValue::Mapping(args),
            );
            return Ok(mapping);
        }
    }
    Err("Failed to get non-empty tool choice after retries".into())
}

pub async fn execute_tool_call(
    tool_name: String,
    args: YamlValue,
    verbose: bool,
    yolo: bool,
    plan_mode: bool,
    ui_mode: bool,
    approval_override: Option<bool>,
) -> Result<String, LoopError> {
    let tools = crate::tools::get_tools(yolo, plan_mode);
    let tool =
        tools.into_iter().find(|t| t.name() == tool_name).ok_or_else(|| {
            LoopError::Other(format!("Unknown tool: {}", tool_name).into())
        })?;
    tool.execute(args, verbose, yolo, ui_mode, approval_override).await
}

pub fn compute_args_str(args_parsed: &YamlValue) -> String {
    if let YamlValue::Mapping(map) = args_parsed {
        map.iter()
            .map(|(k, v)| {
                let key_str = k.as_str().unwrap_or("key");
                if let YamlValue::String(s) = v {
                    format!("{}: '{}'", key_str, s)
                } else {
                    format!(
                        "{}: {}",
                        key_str,
                        serde_yaml::to_string(v).unwrap().trim()
                    )
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    } else {
        "".to_string()
    }
}

pub async fn handle_tool_selection(
    history: &[ChatCompletionRequestMessage],
    model: &str,
    retries: u32,
    max_tokens: u32,
    base_url: &str,
    verbose: bool,
    yolo: bool,
    disable_agents_md: bool,
    plan_mode: bool,
) -> Result<
    (String, YamlValue, String, String),
    Box<dyn std::error::Error + Send + Sync>,
> {
    let mapping = choose_tool(
        history.to_vec(),
        model,
        retries,
        max_tokens,
        base_url,
        verbose,
        yolo,
        disable_agents_md,
        plan_mode,
    )
    .await?;
    let yaml_value = YamlValue::Mapping(mapping.clone());
    let map = &mapping;
    let (key, value) =
        map.iter().next().expect("Mapping should have at least one entry");
    let tool_name = key.as_str().expect("Key should be a string");
    let tool = tool_name.to_string();
    let args_parsed = value.clone();

    let key = crate::tools::get_primary_key(&tool);
    let primary_value = args_parsed
        .as_mapping()
        .and_then(|m| m.get(key))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    if verbose {
        println!(
            "Tool: {}, Args: {}",
            tool,
            serde_yaml::to_string(&args_parsed)?
        );
    }

    let assistant_content =
        serde_yaml::to_string(&yaml_value).unwrap_or_default();
    Ok((tool, args_parsed, primary_value, assistant_content))
}

pub async fn handle_tool_execution(
    tool: String,
    args_parsed: YamlValue,
    verbose: bool,
    yolo: bool,
    plan_mode: bool,
    ui_mode: bool,
    approval_override: Option<bool>,
) -> Result<String, LoopError> {
    execute_tool_call(
        tool,
        args_parsed,
        verbose,
        yolo,
        plan_mode,
        ui_mode,
        approval_override,
    )
    .await
}

pub fn handle_finish_condition(
    tool: &str,
    tool_calls_len: usize,
    max_tool_calls: u32,
    _plan_mode: bool,
) -> bool {
    tool == "finish_task"
        || tool == "finish_planning"
        || (max_tool_calls != 0 && tool_calls_len >= max_tool_calls as usize)
}
