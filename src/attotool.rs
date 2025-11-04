use async_openai::{
    Client,
    types::{
        ChatCompletionRequestAssistantMessage,
        ChatCompletionRequestAssistantMessageContent,
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
        ChatCompletionRequestSystemMessageContent,
        ChatCompletionRequestUserMessage,
        ChatCompletionRequestUserMessageContent, CreateChatCompletionRequest,
    },
};
use serde_json::{Map, Value};
use serde_yaml;
use std::env;
use std::fs;
use std::path::Path;

use crate::response_formats::{
    ToolResponseFormat, parse_tool_response, response_format,
};

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
    tool_response_format: &ToolResponseFormat,
) -> Result<Map<String, Value>, Box<dyn std::error::Error>> {
    let api_key =
        env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY must be set");
    let client = Client::with_config(
        async_openai::config::OpenAIConfig::new()
            .with_api_base(base_url)
            .with_api_key(api_key),
    );

    let tools = crate::tools::get_tools(yolo, plan_mode);
    let tool_names: Vec<serde_json::Value> = tools
        .iter()
        .map(|t| serde_json::Value::String(t.name().to_string()))
        .collect();
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
        tool_response_format,
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

    let response_format_api =
        response_format(tool_response_format, &tool_names);

    let request = CreateChatCompletionRequest {
        model: model.to_string(),
        messages,
        max_completion_tokens: Some(max_tokens),
        response_format: response_format_api,
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
            if let Ok(normalized) =
                parse_tool_response(tool_response_format, trimmed, verbose)
            {
                return Ok(normalized);
            }
            // Fallback: return a map for finish tool
            let finish_tool_name = if plan_mode {
                "finish_planning"
            } else {
                "finish_task"
            };
            let mut args = Map::new();
            args.insert(
                "message".to_string(),
                Value::String(trimmed.to_string()),
            );
            let mut mapping = Map::new();
            mapping.insert(finish_tool_name.to_string(), Value::Object(args));
            return Ok(mapping);
        }
    }
    Err("Failed to get non-empty tool choice after retries".into())
}

pub async fn execute_tool_call(
    tool_name: String,
    args: Value,
    verbose: bool,
    yolo: bool,
    plan_mode: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    let tools = crate::tools::get_tools(yolo, plan_mode);
    let tool = tools
        .into_iter()
        .find(|t| t.name() == tool_name)
        .ok_or_else(|| format!("Unknown tool: {}", tool_name))?;
    tool.execute(args, verbose, yolo).await
}

pub async fn loop_tools_until_finish(
    message: String,
    model: &str,
    retries: u32,
    max_tokens: u32,
    max_tool_calls: u32,
    base_url: &str,
    verbose: bool,
    tool_call_details: bool,
    disable_agents_md: bool,
    yolo: bool,
    continue_task: bool,
    plan_mode: bool,
    tool_response_format: &ToolResponseFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let home = env::var("HOME").expect("HOME not set");
    let history_dir = Path::new(&home).join(".local/share/attotool");
    fs::create_dir_all(&history_dir).ok();
    let history_path = history_dir.join("history.yaml");
    let mut history = Vec::new();
    if continue_task {
        let history_yaml = fs::read_to_string(&history_path)
            .expect("Failed to read history.yaml");
        history = serde_yaml::from_str(&history_yaml)
            .expect("Failed to parse history.yaml");
        history.push(ChatCompletionRequestMessage::User(
            ChatCompletionRequestUserMessage {
                content: ChatCompletionRequestUserMessageContent::Text(message),
                name: None,
            },
        ));
    } else {
        if !disable_agents_md && fs::metadata("AGENTS.md").is_ok() {
            if let Ok(content) = fs::read_to_string("AGENTS.md") {
                let formatted =
                    format!("[read_file path: 'AGENTS.md']\n{}", content);
                history.push(ChatCompletionRequestMessage::User(
                    ChatCompletionRequestUserMessage {
                        content: ChatCompletionRequestUserMessageContent::Text(
                            formatted,
                        ),
                        name: None,
                    },
                ));
            }
        }
        history.push(ChatCompletionRequestMessage::User(
            ChatCompletionRequestUserMessage {
                content: ChatCompletionRequestUserMessageContent::Text(message),
                name: None,
            },
        ));
    }
    let mut tool_calls: Vec<(String, String)> = Vec::new();
    loop {
        let mapping = choose_tool(
            history.clone(),
            model,
            retries,
            max_tokens,
            base_url,
            verbose,
            yolo,
            disable_agents_md,
            plan_mode,
            tool_response_format,
        )
        .await?;
        let json_value = Value::Object(mapping.clone());
        let map = &mapping;
        let (key, value) =
            map.iter().next().expect("Mapping should have at least one entry");
        let tool_name = key.to_string();
        let tool = tool_name.to_string();
        let args_parsed = value.clone();

        let key = match tool.as_str() {
            "execute_shell_command" => "command",
            "read_file" => "path",
            "write_file" => "path",
            "read_lines" => "path",
            "write_lines" => "path",
            "list_files" => "path",
            "ask_for_clarification" => "",
            "describe_to_user" => "",
            "finish_task" => "",
            "finish_planning" => "",
            _ => "",
        };
        let primary_value = if let Value::Object(ref m) = args_parsed {
            m.get(key).and_then(|v| v.as_str()).unwrap_or("")
        } else {
            ""
        }
        .to_string();

        tool_calls.push((tool.clone(), primary_value.clone()));
        if verbose {
            println!(
                "Tool: {}, Args: {}",
                tool,
                serde_json::to_string(&args_parsed)?
            );
        }

        let assistant_content =
            serde_json::to_string(&json_value).unwrap_or_default();
        if !assistant_content.trim().is_empty() {
            history.push(ChatCompletionRequestMessage::Assistant(
                ChatCompletionRequestAssistantMessage {
                    content: Some(
                        ChatCompletionRequestAssistantMessageContent::Text(
                            assistant_content,
                        ),
                    ),
                    name: None,
                    tool_calls: None,
                    ..Default::default()
                },
            ));
        }

        let args_str = if let Value::Object(map) = &args_parsed {
            map.iter()
                .map(|(k, v)| {
                    if let Value::String(s) = v {
                        format!("{}: '{}'", k, s)
                    } else {
                        format!(
                            "{}: {}",
                            k,
                            serde_json::to_string(v).unwrap().trim()
                        )
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
        } else {
            "".to_string()
        };

        let result = match execute_tool_call(
            tool.clone(),
            args_parsed.clone(),
            verbose,
            yolo,
            plan_mode,
        )
        .await
        {
            Ok(result) => result,
            Err(e) => {
                let failure_message =
                    format!("[FAILURE {} {}]", tool, args_str);
                if tool_call_details {
                    println!("Tool call failed: {}", failure_message);
                    println!("Error: {}", e);
                }
                println!("--- [{} {}]", tool, primary_value);
                history.push(ChatCompletionRequestMessage::User(
                    ChatCompletionRequestUserMessage {
                        content: ChatCompletionRequestUserMessageContent::Text(
                            failure_message,
                        ),
                        name: None,
                    },
                ));
                continue;
            }
        };

        let prefixed_result =
            format!("[{} {}]\n{}", tool, primary_value, result);
        if tool_call_details {
            println!(
                "Tool call result: {}",
                prefixed_result.chars().take(500).collect::<String>()
            );
        }
        println!("--- [{} {}]", tool, primary_value);
        history.push(ChatCompletionRequestMessage::User(
            ChatCompletionRequestUserMessage {
                content: ChatCompletionRequestUserMessageContent::Text(
                    prefixed_result,
                ),
                name: None,
            },
        ));

        if tool == "finish_task"
            || tool == "finish_planning"
            || (max_tool_calls != 0
                && tool_calls.len() >= max_tool_calls as usize)
        {
            break;
        }
    }
    println!("--- Task tool usage summary");
    for (tool, arg) in &tool_calls {
        println!("[{} {}]", tool, arg);
    }
    let yaml_content = serde_yaml::to_string(&history).unwrap();
    std::fs::write(&history_path, yaml_content).unwrap();
    Ok(())
}
