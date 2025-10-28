# attotool-rs

A tiny YAML-tool-calling agent built from scratch in Rust.

This project implements a minimalistic agent that uses YAML-formatted tool calls to interact with the local system. It leverages large language models (via OpenRouter API) to choose and execute tools in a structured, human-readable format.

## Features

- **YAML Tool Calling**: All tool interactions are formatted as simple YAML dictionaries
- **Built-in Tools**: Supports shell commands, file operations, user interaction, and task management
- **Loop Mode**: Can execute multiple tool calls in sequence until task completion
- **Single Call Mode**: Execute one tool call and exit
- **Approval Prompts**: User confirmation for potentially destructive operations
- **Conversation History**: Saves interaction history to `history.yaml`
- **Rust Implementation**: Lightweight, fast, and memory-safe

## Installation

1. Ensure you have Rust installed: https://rustup.rs/
2. Clone this repository
3. Build the project: `cargo build --release`
4. Set your OpenRouter API key: `export OPENROUTER_API_KEY=your_key_here`

## Usage

```bash
# Loop mode
cargo run -- --loop "read ./url.txt, fetch that url and describe the result as a markdown document"

# Non-loop mode for single tool call
cargo run -- --input "curl the ubuntu homepage"

# Specify model and other options
cargo run -- --model "openai/gpt-4" --max-tokens 4000 --input "your task here"
```

### CLI Options

- `--model`: LLM model to use (default: openai/gpt-oss-20b)
- `--max-tokens`: Maximum tokens for response (default: 2000)
- `--base-url`: API base URL (default: https://openrouter.ai/api/v1)
- `--input`: Task description
- `--loop`: Enable loop mode for multi-step tasks
- `--retries`: Number of retries for API calls (default: 3)

## Recommended Models

The following models have been tested and have worked at least once with attotool-rs:

- **mistralai/mistral-7b-instruct**
- openai/gpt-oss-20b
- qwen/qwen-2.5-7b-instruct
- z-ai/glm-4-32b

## Available Tools

- `execute_shell_command`: Run shell commands with arguments
- `read_file`: Read file contents
- `write_file`: Write content to files
- `finish_task`: Mark task as completed
- `ask_for_clarification`: Request user input
- `describe_to_user`: Provide descriptions or responses

## Architecture

The agent works by:

1. Sending the task and available tools to an LLM
2. Receiving a YAML-formatted tool call response
3. Parsing and executing the tool locally
4. Feeding results back to continue the conversation

All communication with the LLM is through simple YAML structures, making the system transparent and debuggable.

## Dependencies

- `async-openai`: OpenAI API client
- `serde_yaml`: YAML serialization
- `tokio`: Async runtime
- `clap`: CLI argument parsing