use serde_json::Value;
use std::fs;
use std::io::{self, Write};
use std::process;

#[derive(Clone)]
pub enum Tool {
    ExecuteShellCommand,
    ReadFile,
    WriteFile,
    FinishTask,
    AskForClarification,
    DescribeToUser,
}

impl Tool {
    pub fn name(&self) -> &str {
        match self {
            Tool::ExecuteShellCommand => "execute_shell_command",
            Tool::ReadFile => "read_file",
            Tool::WriteFile => "write_file",
            Tool::FinishTask => "finish_task",
            Tool::AskForClarification => "ask_for_clarification",
            Tool::DescribeToUser => "describe_to_user",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            Tool::ExecuteShellCommand => {
                "Executes a command with arguments on the zsh shell - includes common tools like ls, pwd, curl, cat, mkdir"
            }
            Tool::ReadFile => "Reads a file on the local filesystem",
            Tool::WriteFile => "Writes a file on the local filesystem",
            Tool::FinishTask => {
                "Marks the assigned task as completed, with a completion message"
            }
            Tool::AskForClarification => {
                "Allows the assistant to ask the user for clarification on a point of interest"
            }
            Tool::DescribeToUser => {
                "Provides a description or response to the user"
            }
        }
    }

    pub fn parameters(&self) -> Vec<(String, String)> {
        match self {
            Tool::ExecuteShellCommand => vec![
                ("command".to_string(), "string".to_string()),
                ("args".to_string(), "string".to_string()),
            ],
            Tool::ReadFile => vec![("path".to_string(), "string".to_string())],
            Tool::WriteFile => vec![
                ("path".to_string(), "string".to_string()),
                ("content".to_string(), "string".to_string()),
            ],
            Tool::FinishTask => {
                vec![("message".to_string(), "string".to_string())]
            }
            Tool::AskForClarification => {
                vec![("question".to_string(), "string".to_string())]
            }
            Tool::DescribeToUser => {
                vec![("description".to_string(), "string".to_string())]
            }
        }
    }

    pub async fn execute(
        &self,
        args: Value,
        verbose: bool,
    ) -> Result<String, Box<dyn std::error::Error>> {
        match self {
            Tool::ExecuteShellCommand => {
                execute_shell_command(args, verbose).await
            }
            Tool::ReadFile => execute_read_file(args, verbose).await,
            Tool::WriteFile => execute_write_file(args, verbose).await,
            Tool::FinishTask => execute_finish_task(args, verbose).await,
            Tool::AskForClarification => {
                execute_ask_for_clarification(args, verbose).await
            }
            Tool::DescribeToUser => {
                execute_describe_to_user(args, verbose).await
            }
        }
    }

    pub fn format(&self) -> String {
        let params = self
            .parameters()
            .iter()
            .map(|(p, t)| format!("  {}: {}", p, t))
            .collect::<Vec<_>>()
            .join("\n");
        format!("{}: '{}'\n{}", self.name(), self.description(), params)
    }
}

fn prompt_approval(prompt: &str, _verbose: bool) -> bool {
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

async fn execute_shell_command(
    args: Value,
    verbose: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    let command = args["command"].as_str().unwrap_or("");
    let args_str = args["args"].as_str().unwrap_or("");
    if !prompt_approval(
        &format!(
            "Do you want to run this command: `{} {}` ? (Y/n): ",
            command, args_str
        ),
        verbose,
    ) {
        return Ok("Command execution cancelled.".to_string());
    }
    let output = process::Command::new("zsh")
        .arg("-c")
        .arg(format!("{} {}", command, args_str))
        .output()
        .expect("Failed to execute command");
    let mut result = format!("{}", String::from_utf8_lossy(&output.stdout));
    if !output.stderr.is_empty() {
        result.push_str(&format!(
            "\nStderr: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(result)
}

async fn execute_read_file(
    args: Value,
    _verbose: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    let path = args["path"].as_str().unwrap_or("");
    match fs::read_to_string(path) {
        Ok(content) => Ok(content),
        Err(e) => Ok(format!("Error reading file: {}", e)),
    }
}

async fn execute_write_file(
    args: Value,
    verbose: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    let path = args["path"].as_str().unwrap_or("");
    let content = args["content"].as_str().unwrap_or("");
    if !prompt_approval(
        &format!("Do you want to write to file: {}? (Y/n): ", path),
        verbose,
    ) {
        return Ok("File write cancelled.".to_string());
    }
    match fs::write(path, content) {
        Ok(_) => Ok("File written successfully".to_string()),
        Err(e) => Ok(format!("Error writing file: {}", e)),
    }
}

async fn execute_finish_task(
    args: Value,
    _verbose: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    let message = args["message"].as_str().unwrap_or("");
    println!("{}", format!("Task completed: {}", message));
    Ok(format!("Task completed: {}", message))
}

async fn execute_ask_for_clarification(
    args: Value,
    _verbose: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    let question = args["question"].as_str().unwrap_or("");
    println!("{}", question);
    let mut answer = String::new();
    io::stdin().read_line(&mut answer).unwrap();
    Ok(answer.trim().to_string())
}

async fn execute_describe_to_user(
    args: Value,
    _verbose: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    let description = args["description"].as_str().unwrap_or("");
    println!("{}", format!("Description: {}", description));
    Ok(format!("Description: {}", description))
}

pub fn get_tools() -> Vec<Tool> {
    vec![
        Tool::ExecuteShellCommand,
        Tool::ReadFile,
        Tool::WriteFile,
        Tool::FinishTask,
        Tool::AskForClarification,
        Tool::DescribeToUser,
    ]
}
