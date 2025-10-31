use async_openai::types::{
    ChatCompletionRequestAssistantMessage,
    ChatCompletionRequestAssistantMessageContent, ChatCompletionRequestMessage,
    ChatCompletionRequestUserMessage, ChatCompletionRequestUserMessageContent,
};
use std::fs;

use crate::output::Output;

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
    ui_mode: bool,
    approval_override: Option<bool>,
    output: &mut dyn Output,
    tool_calls: &mut Vec<(String, String)>,
) -> Result<(), crate::tool_handler::LoopError> {
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
    loop {
        let (tool, args_parsed, primary_value, assistant_content) =
            crate::tool_handler::handle_tool_selection(
                &history,
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
        tool_calls.push((tool.clone(), primary_value.clone()));
        let args_str = crate::tool_handler::compute_args_str(&args_parsed);
        output.print_tool_call(&tool, &primary_value);
        let result = crate::tool_handler::handle_tool_execution(
            tool.clone(),
            args_parsed,
            verbose,
            yolo,
            plan_mode,
            ui_mode,
            approval_override,
        )
        .await;
        match result {
            Ok(res) => {
                output.print_result(&tool, &primary_value, &res);
                let prefixed_result =
                    format!("[{} {}]\n{}", tool, primary_value, res);
                history.push(ChatCompletionRequestMessage::User(
                    ChatCompletionRequestUserMessage {
                        content: ChatCompletionRequestUserMessageContent::Text(
                            prefixed_result,
                        ),
                        name: None,
                    },
                ));
            }
            Err(crate::tool_handler::LoopError::Prompt(prompt_type)) => {
                return Err(crate::tool_handler::LoopError::Prompt(
                    prompt_type,
                ));
            }
            Err(crate::tool_handler::LoopError::Other(e)) => {
                output.print_failure(
                    &tool,
                    &primary_value,
                    &args_str,
                    &e.to_string(),
                );
                let failure_message =
                    format!("[FAILURE {} {}]", tool, args_str);
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
        }
        if crate::tool_handler::handle_finish_condition(
            &tool,
            tool_calls.len(),
            max_tool_calls,
            plan_mode,
        ) {
            break;
        }
    }
    output.print_summary(&tool_calls);
    let yaml_content = serde_yaml::to_string(&history).unwrap();
    std::fs::write("./history.yaml", yaml_content).unwrap();
    Ok(())
}
