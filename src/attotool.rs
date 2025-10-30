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
use serde_yaml::{Mapping, Value as YamlValue};
use std::env;
use std::fs;

fn parse_and_normalize_yaml(
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

pub async fn choose_tool(
    history: Vec<ChatCompletionRequestMessage>,
    model: &str,
    retries: u32,
    max_tokens: u32,
    base_url: &str,
    verbose: bool,
    yolo: bool,
    disable_agents_md: bool,
) -> Result<Mapping, Box<dyn std::error::Error>> {
    let api_key =
        env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY must be set");
    let client = Client::with_config(
        async_openai::config::OpenAIConfig::new()
            .with_api_base(base_url)
            .with_api_key(api_key),
    );

    let tools = crate::tools::get_tools(yolo);
    let available_tools_text =
        tools.iter().map(|t| t.format()).collect::<Vec<_>>().join("\n");
    let current_dir = std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("unknown"));
    let agents_md_preamble = if !disable_agents_md
        && fs::metadata("AGENTS.md").is_ok()
    {
        "\n\nAGENTS.md is an open format for guiding tool-calling agents, providing project-specific instructions like build steps, code style, and conventions to help AI agents work effectively on the codebase."
    } else {
        ""
    };

    let system_message = ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
        content: ChatCompletionRequestSystemMessageContent::Text(
            format!("You are a tool calling agent who responds with a single-item YAML dictionary. You ONLY respond in tool calls, one per message, with nothing before or after the YAML. Remember to format your strings as valid yaml (either escaping newlines or using pipe strings). Respond with the tool name and its arguments in the format:

tool_name:
  arg1: 'value1'
  arg2: value2
        
You will be given a user message which defines a task, and your job is to choose which tool would be most appropriate to use to accomplish or make progress on the task, and provide the necessary arguments for that tool. The tool call will then be executed by the user and the result returned. You will then choose another tool to continue the task.

If the task is finished, use the finish_task tool. If you need additional information, use ask_for_clarification

The current working directory is {}{}

Your available tools:

{}

An example of appropriate response formatting:

read_file:
  path: '/some/file.txt'", current_dir.display(), agents_md_preamble, available_tools_text).to_string()),
        name: None,
    });

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
            if let Ok(normalized) =
                parse_and_normalize_yaml(yaml_candidate, verbose)
            {
                return Ok(normalized);
            }
            // Fallback: return a mapping for finish_task
            let mut args = Mapping::new();
            args.insert(
                YamlValue::String("message".to_string()),
                YamlValue::String(trimmed.to_string()),
            );
            let mut mapping = Mapping::new();
            mapping.insert(
                YamlValue::String("finish_task".to_string()),
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
) -> Result<String, Box<dyn std::error::Error>> {
    let tools = crate::tools::get_tools(yolo);
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
) -> Result<(), Box<dyn std::error::Error>> {
    let mut history = Vec::new();
    if continue_task {
        let history_yaml = fs::read_to_string("history.yaml")
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
        )
        .await?;
        let yaml_value = YamlValue::Mapping(mapping.clone());
        let map = &mapping;
        let (key, value) =
            map.iter().next().expect("Mapping should have at least one entry");
        let tool_name = key.as_str().expect("Key should be a string");
        let tool = tool_name.to_string();
        let args_parsed = value.clone();

        let key = match tool.as_str() {
            "execute_shell_command" => "command",
            "read_file" => "path",
            "write_file" => "path",
            "list_files" => "path",
            "ask_for_clarification" => "",
            "describe_to_user" => "",
            "finish_task" => "",
            _ => "",
        };
        let primary_value = args_parsed
            .as_mapping()
            .and_then(|m| m.get(key))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        tool_calls.push((tool.clone(), primary_value.clone()));
        if verbose {
            println!(
                "Tool: {}, Args: {}",
                tool,
                serde_yaml::to_string(&args_parsed)?
            );
        }

        let assistant_content =
            serde_yaml::to_string(&yaml_value).unwrap_or_default();
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

        let args_str = if let YamlValue::Mapping(map) = &args_parsed {
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
        };

        let result = match execute_tool_call(
            tool.clone(),
            args_parsed.clone(),
            verbose,
            yolo,
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
                    println!("---");
                }
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
            println!("---");
        }
        history.push(ChatCompletionRequestMessage::User(
            ChatCompletionRequestUserMessage {
                content: ChatCompletionRequestUserMessageContent::Text(
                    prefixed_result,
                ),
                name: None,
            },
        ));

        if tool == "finish_task"
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
    std::fs::write("./history.yaml", yaml_content).unwrap();
    Ok(())
}
