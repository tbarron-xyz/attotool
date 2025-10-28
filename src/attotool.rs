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
use serde_json::Value;
use serde_yaml::{Mapping, Value as YamlValue};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::process;

pub async fn choose_tool(
    history: Vec<ChatCompletionRequestMessage>,
    model: &str,
    retries: u32,
) -> Result<String, Box<dyn std::error::Error>> {
    let api_key =
        env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY must be set");
    let client = Client::with_config(
        async_openai::config::OpenAIConfig::new()
            .with_api_base("https://openrouter.ai/api/v1")
            .with_api_key(api_key),
    );

    let available_tools_text = format!("\
execute_shell_command: 'Executes a command with arguments on the zsh shell - includes common tools like ls, pwd, curl, cat, mkdir'
  command: string
  args: string
read_file: 'Reads a file on the local filesystem'
  path: string
write_file: 'Writes a file on the local filesystem'
  path: string
  content: string
finish_task: 'Marks the assigned task as completed, with a completion message'
  message: string
ask_for_clarification: 'Allows the assistant to ask the user for clarification on a point of interest'
  question: string
describe_to_user: 'Provides a description or response to the user'
  description: string");

    let system_message = ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
        content: ChatCompletionRequestSystemMessageContent::Text(
            format!("You are a tool calling agent who responds with a single-item YAML dictionary. You ONLY respond in tool calls, one per message, with nothing before or after the YAML. Remember to format your strings as valid yaml (either escaping newlines or using pipe strings). Respond with the tool name and its arguments in the format:

tool_name:
  arg1: 'value1'
  arg2: value2
        
You will be given a user message which defines a task, and your job is to choose which tool would be most appropriate to use to accomplish or make progress on the task, and provide the necessary arguments for that tool. The tool call will then be executed by the user and the result returned. You will then choose another tool to continue the task.

If the task is finished, use the finish_task tool. If you need additional information, use ask_for_clarification

Your available tools:

{}

An example of appropriate response formatting:

read_file:
  path: '/some/file.txt'", available_tools_text).to_string()),
        name: None,
    });

    let mut messages = vec![system_message];
    messages.extend(history);

    let request = CreateChatCompletionRequest {
        model: model.to_string(),
        messages,
        ..Default::default()
    };

    for attempt in 0..retries {
        println!("Sending openai request");
        let response = client.chat().create(request.clone()).await?;
        let choice = response.choices.first().ok_or("No response")?;
        let content = choice.message.content.as_ref().ok_or("No content")?;
        let trimmed = content.trim();
        println!(
            "API Response Content (choose_tool, attempt {}):\n{}",
            attempt + 1,
            content
        );
        if !trimmed.is_empty() {
            // First, try parsing the entire trimmed response as YAML
            if let Ok(value) = serde_yaml::from_str::<YamlValue>(trimmed) {
                if let YamlValue::Mapping(mapping) = value {
                    if mapping.len() > 1 {
                        println!(
                            "Removed {} additional tool(s) from multi-tool response",
                            mapping.len() - 1
                        );
                        let mut new_mapping = Mapping::new();
                        if let Some((key, val)) = mapping.iter().next() {
                            new_mapping.insert(key.clone(), val.clone());
                        }
                        let new_yaml = serde_yaml::to_string(
                            &YamlValue::Mapping(new_mapping),
                        )
                        .unwrap();
                        return Ok(new_yaml.trim().to_string());
                    } else {
                        return Ok(trimmed.to_string());
                    }
                }
            }
            // If parsing the whole failed, try splitting by \n\n and parse the first part
            let yaml_candidate = if let Some(pos) = trimmed.find("\n\n") {
                &trimmed[..pos]
            } else {
                trimmed
            };
            if let Ok(value) = serde_yaml::from_str::<YamlValue>(yaml_candidate)
            {
                if let YamlValue::Mapping(mapping) = value {
                    if mapping.len() > 1 {
                        println!(
                            "Removed {} additional tool(s) from multi-tool response",
                            mapping.len() - 1
                        );
                        let mut new_mapping = Mapping::new();
                        if let Some((key, val)) = mapping.iter().next() {
                            new_mapping.insert(key.clone(), val.clone());
                        }
                        let new_yaml = serde_yaml::to_string(
                            &YamlValue::Mapping(new_mapping),
                        )
                        .unwrap();
                        return Ok(new_yaml.trim().to_string());
                    } else {
                        return Ok(yaml_candidate.to_string());
                    }
                }
            }
            return Ok(trimmed.to_string());
        }
    }
    Err("Failed to get non-empty tool choice after retries".into())
}

fn prompt_approval(prompt: &str) -> bool {
    println!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let input = input.trim().to_lowercase();
    if input.is_empty() {
        return true;
    }
    input == "y"
}

pub async fn execute_tool_call(
    tool_name: String,
    args: Value,
) -> Result<String, Box<dyn std::error::Error>> {
    match tool_name.as_str() {
        "execute_shell_command" => {
            let command = args["command"].as_str().unwrap_or("");
            let args_str = args["args"].as_str().unwrap_or("");
            if !prompt_approval(&format!(
                "Do you want to run this command: `{} {}` ? (Y/n): ",
                command, args_str
            )) {
                return Ok("Command execution cancelled.".to_string());
            }
            let output = process::Command::new("zsh")
                .arg("-c")
                .arg(format!("{} {}", command, args_str))
                .output()
                .expect("Failed to execute command");
            let mut result =
                format!("{}", String::from_utf8_lossy(&output.stdout));
            if !output.stderr.is_empty() {
                result.push_str(&format!(
                    "\nStderr: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
            Ok(result)
        }
        "read_file" => {
            let path = args["path"].as_str().unwrap_or("");
            match fs::read_to_string(path) {
                Ok(content) => Ok(content),
                Err(e) => Ok(format!("Error reading file: {}", e)),
            }
        }
        "write_file" => {
            let path = args["path"].as_str().unwrap_or("");
            let content = args["content"].as_str().unwrap_or("");
            if !prompt_approval(&format!(
                "Do you want to write to file: {}? (Y/n): ",
                path
            )) {
                return Ok("File write cancelled.".to_string());
            }
            match fs::write(path, content) {
                Ok(_) => Ok("File written successfully".to_string()),
                Err(e) => Ok(format!("Error writing file: {}", e)),
            }
        }
        "finish_task" => {
            let message = args["message"].as_str().unwrap_or("");
            Ok(format!("Task completed: {}", message))
        }
        "describe_to_user" => {
            let description = args["description"].as_str().unwrap_or("");
            Ok(format!("Description: {}", description))
        }
        "ask_for_clarification" => {
            let question = args["question"].as_str().unwrap_or("");
            println!("{}", question);
            let mut answer = String::new();
            io::stdin().read_line(&mut answer).unwrap();
            Ok(answer.trim().to_string())
        }
        _ => Ok(format!("Unknown tool: {}", tool_name)),
    }
}

pub async fn loop_tools_until_finish(
    message: String,
    model: &str,
    retries: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut history = vec![ChatCompletionRequestMessage::User(
        ChatCompletionRequestUserMessage {
            content: ChatCompletionRequestUserMessageContent::Text(message),
            name: None,
        },
    )];
    loop {
        let response = choose_tool(history.clone(), model, retries).await?;
        let yaml_value: YamlValue = match serde_yaml::from_str(&response) {
            Ok(v) => v,
            Err(_) => {
                let tool = "finish_task".to_string();
                let args_parsed = serde_json::json!({"message": response});
                println!(
                    "Tool: {}, Args: {}",
                    tool,
                    serde_yaml::to_string(&args_parsed)?
                );
                let assistant_content = response;
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
                let result =
                    execute_tool_call(tool.clone(), args_parsed.clone())
                        .await?;
                let args_str =
                    if let serde_json::Value::Object(obj) = &args_parsed {
                        obj.iter()
                            .map(|(k, v)| {
                                if let serde_json::Value::String(s) = v {
                                    format!("{}: '{}'", k, s)
                                } else {
                                    format!("{}: {}", k, v.to_string())
                                }
                            })
                            .collect::<Vec<_>>()
                            .join(" ")
                    } else {
                        "".to_string()
                    };
                let prefixed_result =
                    format!("[{} {}]\n{}", tool, args_str, result);
                println!(
                    "Tool call result: {}",
                    prefixed_result.chars().take(500).collect::<String>()
                );
                println!("---");
                history.push(ChatCompletionRequestMessage::User(
                    ChatCompletionRequestUserMessage {
                        content: ChatCompletionRequestUserMessageContent::Text(
                            prefixed_result,
                        ),
                        name: None,
                    },
                ));
                if tool == "finish_task" {
                    break;
                }
                continue;
            }
        };
        let (tool, args_parsed) =
            if let YamlValue::Mapping(mapping) = yaml_value {
                if let Some((key, value)) = mapping.into_iter().next() {
                    if let YamlValue::String(tool_name) = key {
                        (tool_name, serde_json::to_value(value)?)
                    } else {
                        (
                            "finish_task".to_string(),
                            serde_json::json!({"message": response}),
                        )
                    }
                } else {
                    (
                        "finish_task".to_string(),
                        serde_json::json!({"message": response}),
                    )
                }
            } else {
                (
                    "finish_task".to_string(),
                    serde_json::json!({"message": response}),
                )
            };
        println!(
            "Tool: {}, Args: {}",
            tool,
            serde_yaml::to_string(&args_parsed)?
        );

        let assistant_content = response;
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

        let result =
            execute_tool_call(tool.clone(), args_parsed.clone()).await?;

        let args_str = if let serde_json::Value::Object(obj) = &args_parsed {
            obj.iter()
                .map(|(k, v)| {
                    if let serde_json::Value::String(s) = v {
                        format!("{}: '{}'", k, s)
                    } else {
                        format!("{}: {}", k, v.to_string())
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
        } else {
            "".to_string()
        };
        let prefixed_result = format!("[{} {}]\n{}", tool, args_str, result);
        println!(
            "Tool call result: {}",
            prefixed_result.chars().take(500).collect::<String>()
        );
        println!("---");
        history.push(ChatCompletionRequestMessage::User(
            ChatCompletionRequestUserMessage {
                content: ChatCompletionRequestUserMessageContent::Text(
                    prefixed_result,
                ),
                name: None,
            },
        ));

        if tool == "finish_task" {
            break;
        }
    }
    let yaml_content = serde_yaml::to_string(&history).unwrap();
    std::fs::write("./history.yaml", yaml_content).unwrap();
    Ok(())
}

pub async fn one_function_call(
    message: String,
    model: &str,
    retries: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut history = vec![ChatCompletionRequestMessage::User(
        ChatCompletionRequestUserMessage {
            content: ChatCompletionRequestUserMessageContent::Text(message),
            name: None,
        },
    )];
    let response = choose_tool(history.clone(), model, retries).await?;
    let (tool, args_parsed) = match serde_yaml::from_str(&response) {
        Ok(yaml_value) => {
            if let YamlValue::Mapping(mapping) = yaml_value {
                if let Some((key, value)) = mapping.into_iter().next() {
                    if let YamlValue::String(tool_name) = key {
                        (tool_name, serde_json::to_value(value).unwrap())
                    } else {
                        (
                            "finish_task".to_string(),
                            serde_json::json!({"message": response}),
                        )
                    }
                } else {
                    (
                        "finish_task".to_string(),
                        serde_json::json!({"message": response}),
                    )
                }
            } else {
                (
                    "finish_task".to_string(),
                    serde_json::json!({"message": response}),
                )
            }
        }
        Err(_) => (
            "finish_task".to_string(),
            serde_json::json!({"message": response}),
        ),
    };
    println!(
        "Tool: {}, Args: {}",
        tool,
        serde_yaml::to_string(&args_parsed).unwrap()
    );
    history.push(ChatCompletionRequestMessage::Assistant(
        ChatCompletionRequestAssistantMessage {
            content: Some(ChatCompletionRequestAssistantMessageContent::Text(
                response.clone(),
            )),
            name: None,
            tool_calls: None,
            ..Default::default()
        },
    ));
    let result = execute_tool_call(tool.clone(), args_parsed.clone()).await?;
    println!("{}", result.chars().take(500).collect::<String>());
    let args_str = if let serde_json::Value::Object(obj) = &args_parsed {
        obj.iter()
            .map(|(k, v)| {
                if let serde_json::Value::String(s) = v {
                    format!("{}: '{}'", k, s)
                } else {
                    format!("{}: {}", k, v.to_string())
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    } else {
        "".to_string()
    };
    let prefixed_result = format!("[{} {}]\n{}", tool, args_str, result);
    history.push(ChatCompletionRequestMessage::User(
        ChatCompletionRequestUserMessage {
            content: ChatCompletionRequestUserMessageContent::Text(
                prefixed_result,
            ),
            name: None,
        },
    ));
    let yaml_content = serde_yaml::to_string(&history).unwrap();
    std::fs::write("./history.yaml", yaml_content).unwrap();
    Ok(())
}
