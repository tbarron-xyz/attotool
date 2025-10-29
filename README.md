# attotool

A tiny YAML-tool-calling agent built from scratch in Rust.

This project implements a minimalistic agent that uses YAML-formatted tool calls to interact with the local system. It leverages large language models (via OpenAI-compatible API, default OpenRouter) to choose and execute tools in a compact, structured, human-readable format.

## Features

- **YAML Tool Calling**: All tool interactions are formatted as simple YAML dictionaries
- **Built-in Tools**: Supports shell commands, file operations, user interaction, and task management
- **Optional Tool Call Limit**: Control the maximum number of tool calls (0 for infinite loop, 1 for single call, etc.)
- **Approval Prompts**: User confirmation for potentially destructive operations
- **Conversation History**: Saves interaction history to `history.yaml`
- **Rust Implementation**: Lightweight, fast, and memory-safe

## Installation

1. Ensure you have Rust installed: https://rustup.rs/
2. Clone this repository
3. Build the project: `cargo build --release`
4. Set your OpenRouter (or OpenAI, to the same env var) API key: `export OPENROUTER_API_KEY=your_key_here`

## Usage

```bash
# Infinite loop (default)
cargo run -- --input "read ./url.txt, fetch that url and describe the result as a markdown document"

# Single tool call
cargo run -- --max-tool-calls 1 --input "curl the ubuntu homepage"

# Specify model and other options
cargo run -- --model "openai/gpt-4" --max-tokens 4000 --input "your task here"
```

### CLI Options

- `--model`: LLM model to use (default: z-ai/glm-4-32b)
- `--max-tokens`: Maximum tokens for response (default: 2000)
- `--base-url`: API base URL (default: https://openrouter.ai/api/v1, use https://api.openai.com/v1 for OpenAI)
- `--input`: Task description
- `--max-tool-calls`: Maximum number of tool calls (default: 0 for infinite)
- `--retries`: Number of retries for API calls (default: 3)
- `--verbose`: Enable detailed output including tool calls and API responses
- `--tool-call-details`: Show detailed tool call results and execution output

## Recommended Models

The following models have been tested and have worked at least once with attotool:

- **z-ai/glm-4-32b**
- mistralai/mistral-7b-instruct
- google/gemma-3-27b-it
- openai/gpt-oss-20b
- openai/gpt-4o-mini
- qwen/qwen-2.5-7b-instruct
- qwen/qwen-2.5-72b-instruct
- mistralai/mistral-nemo
- mistralai/mistral-small-3.1-24b-instruct
- mistralai/devstral-small-2505
- deepseek/deepseek-chat-v3-0324
- x-ai/grok-code-fast-1
- x-ai/grok-4-fast
- x-ai/grok-3-mini

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
4. Return tool call results to LLM to continue the conversation

All LLM tool calls are simple YAML structures, optimizing token consumption and simplifying grammar.

```yaml
execute_shell_command:
  command: 'ls'
  args: '-la'
```

## Output and Summary

Upon task completion (either via the `finish_task` tool or reaching the `max_tool_calls` limit), the agent prints a summary of all executed tool calls in bracketed format (e.g., `[execute_shell_command]`, `[read_file]`). Additionally, the full conversation history, including all tool interactions and responses, is saved to `history.yaml` for review and debugging. This provides transparency into the agent's decision-making process and allows users to audit the sequence of actions taken.