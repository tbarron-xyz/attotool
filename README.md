# attotool

A tiny YAML-tool-calling agent built from scratch in Rust.

[![build docker image](https://github.com/tbarron-xyz/attotool/actions/workflows/build-docker.yml/badge.svg)](https://github.com/tbarron-xyz/attotool/actions/workflows/build-docker.yml)
[![build rust](https://github.com/tbarron-xyz/attotool/actions/workflows/ci-build.yml/badge.svg)](https://github.com/tbarron-xyz/attotool/actions/workflows/ci-build.yml)

attotool is a minimalistic agent that uses YAML-formatted tool calls to interact with the local system. It lets large language models choose and execute tools in a loop until task completion, in a compact, structured, human-readable format.

## Eval Results

`Read at least 10 files in the repo and summarize your findings`

[![Eval Execution - gpt-4o-mini](https://github.com/tbarron-xyz/attotool/actions/workflows/eval-execution-gpt-4o-mini.yml/badge.svg)](https://github.com/tbarron-xyz/attotool/actions/workflows/eval-execution-gpt-4o-mini.yml)
[![Eval Execution - grok-4-fast](https://github.com/tbarron-xyz/attotool/actions/workflows/eval-execution-grok-4-fast.yml/badge.svg)](https://github.com/tbarron-xyz/attotool/actions/workflows/eval-execution-grok-4-fast.yml)
[![Eval Execution - grok-code-fast-1](https://github.com/tbarron-xyz/attotool/actions/workflows/eval-execution-grok-code-fast-1.yml/badge.svg)](https://github.com/tbarron-xyz/attotool/actions/workflows/eval-execution-grok-code-fast-1.yml)

| Criteria \ Model | grok-4-fast | gpt-4o-mini | grok-code-fast-1 |
|------------------|-------------|-------------|------------------|
| Finished task | [![Finished task - grok-4-fast](https://github.com/tbarron-xyz/attotool/actions/workflows/eval-finished-task-grok-4-fast.yml/badge.svg)](https://github.com/tbarron-xyz/attotool/actions/workflows/eval-finished-task-grok-4-fast.yml) | [![Finished task - gpt-4o-mini](https://github.com/tbarron-xyz/attotool/actions/workflows/eval-finished-task-gpt-4o-mini.yml/badge.svg)](https://github.com/tbarron-xyz/attotool/actions/workflows/eval-finished-task-gpt-4o-mini.yml) | [![Finished task - grok-code-fast-1](https://github.com/tbarron-xyz/attotool/actions/workflows/eval-finished-task-grok-code-fast-1.yml/badge.svg)](https://github.com/tbarron-xyz/attotool/actions/workflows/eval-finished-task-grok-code-fast-1.yml) |
| Read 3 files | [![Read 3 files - grok-4-fast](https://github.com/tbarron-xyz/attotool/actions/workflows/eval-read-3-files-grok-4-fast.yml/badge.svg)](https://github.com/tbarron-xyz/attotool/actions/workflows/eval-read-3-files-grok-4-fast.yml) | [![Read 3 files - gpt-4o-mini](https://github.com/tbarron-xyz/attotool/actions/workflows/eval-read-3-files-gpt-4o-mini.yml/badge.svg)](https://github.com/tbarron-xyz/attotool/actions/workflows/eval-read-3-files-gpt-4o-mini.yml) | [![Read 3 files - grok-code-fast-1](https://github.com/tbarron-xyz/attotool/actions/workflows/eval-read-3-files-grok-code-fast-1.yml/badge.svg)](https://github.com/tbarron-xyz/attotool/actions/workflows/eval-read-3-files-grok-code-fast-1.yml) |
| Read 8 files | [![Read 8 files - grok-4-fast](https://github.com/tbarron-xyz/attotool/actions/workflows/eval-read-8-files-grok-4-fast.yml/badge.svg)](https://github.com/tbarron-xyz/attotool/actions/workflows/eval-read-8-files-grok-4-fast.yml) | [![Read 8 files - gpt-4o-mini](https://github.com/tbarron-xyz/attotool/actions/workflows/eval-read-8-files-gpt-4o-mini.yml/badge.svg)](https://github.com/tbarron-xyz/attotool/actions/workflows/eval-read-8-files-gpt-4o-mini.yml) | [![Read 8 files - grok-code-fast-1](https://github.com/tbarron-xyz/attotool/actions/workflows/eval-read-8-files-grok-code-fast-1.yml/badge.svg)](https://github.com/tbarron-xyz/attotool/actions/workflows/eval-read-8-files-grok-code-fast-1.yml) |

---

`read the url at ./url.txt , fetch that url, and write a yaml summary of its contents to ./summary.yaml`

[![Eval Execution Fetch URL - glm-4-32b](https://github.com/tbarron-xyz/attotool/actions/workflows/eval-execution-fetch-url-glm-4-32b.yml/badge.svg)](https://github.com/tbarron-xyz/attotool/actions/workflows/eval-execution-fetch-url-glm-4-32b.yml)
[![Eval Execution Fetch URL - gpt-4o-mini](https://github.com/tbarron-xyz/attotool/actions/workflows/eval-execution-fetch-url-gpt-4o-mini.yml/badge.svg)](https://github.com/tbarron-xyz/attotool/actions/workflows/eval-execution-fetch-url-gpt-4o-mini.yml)
[![Eval Execution Fetch URL - mistral-small-3.1-24b](https://github.com/tbarron-xyz/attotool/actions/workflows/eval-execution-fetch-url-mistral-small-3.1-24b.yml/badge.svg)](https://github.com/tbarron-xyz/attotool/actions/workflows/eval-execution-fetch-url-mistral-small-3.1-24b.yml)

| Criteria \ Model | glm-4-32b | gpt-4o-mini | mistral-small-3.1-24b |
|------------------|-----------|-------------|-----------------------|
| Used curl or wget | [![Used curl or wget - glm-4-32b](https://github.com/tbarron-xyz/attotool/actions/workflows/eval-used-curl-wget-glm-4-32b.yml/badge.svg)](https://github.com/tbarron-xyz/attotool/actions/workflows/eval-used-curl-wget-glm-4-32b.yml) | [![Used curl or wget - gpt-4o-mini](https://github.com/tbarron-xyz/attotool/actions/workflows/eval-used-curl-wget-gpt-4o-mini.yml/badge.svg)](https://github.com/tbarron-xyz/attotool/actions/workflows/eval-used-curl-wget-gpt-4o-mini.yml) | [![Used curl or wget - mistral-small-3.1-24b](https://github.com/tbarron-xyz/attotool/actions/workflows/eval-used-curl-wget-mistral-small-3.1-24b.yml/badge.svg)](https://github.com/tbarron-xyz/attotool/actions/workflows/eval-used-curl-wget-mistral-small-3.1-24b.yml) |
| summary.yaml valid YAML | [![Valid YAML - glm-4-32b](https://github.com/tbarron-xyz/attotool/actions/workflows/eval-valid-yaml-glm-4-32b.yml/badge.svg)](https://github.com/tbarron-xyz/attotool/actions/workflows/eval-valid-yaml-glm-4-32b.yml) | [![Valid YAML - gpt-4o-mini](https://github.com/tbarron-xyz/attotool/actions/workflows/eval-valid-yaml-gpt-4o-mini.yml/badge.svg)](https://github.com/tbarron-xyz/attotool/actions/workflows/eval-valid-yaml-gpt-4o-mini.yml) | [![Valid YAML - mistral-small-3.1-24b](https://github.com/tbarron-xyz/attotool/actions/workflows/eval-valid-yaml-mistral-small-3.1-24b.yml/badge.svg)](https://github.com/tbarron-xyz/attotool/actions/workflows/eval-valid-yaml-mistral-small-3.1-24b.yml) |

## Available Tools

- `execute_shell_command`: Run shell commands with arguments. 🟢 **Requires explicit user confirmation.**
- `read_file`: Read file contents
- `write_file`: Write content to file. 🟢 **Requires explicit user confirmation.**
- `finish_task`: Mark task as completed
- `ask_for_clarification`: Request user input
- `describe_to_user`: Provide descriptions or responses

## Features

- **Plan Mode**: Enable read-only phase with `--plan` / `-p` flag, encouraging analysis and planning and forbidding all modifications
- **Approval Prompts**: User confirmation for potentially destructive operations (`write_file`, `execute_shell_command`)
- **AGENTS.md Support**: Automatically loads ./AGENTS.md as the first user message
- **Conversation History**: Saves interaction history to `history.yaml`
- **attotool.yaml Configuration**: Load model setting from `~/.config/attotool/attotool.yaml`
- **System Prompt Customization**: Load user-defined system prompt section overrides from `~/.config/attotool/system_prompt.yaml`, allowing customization of agent behavior while preserving defaults.
- **Evals in GH Actions**: Automated workflows for evaluating agent performance across multiple language models on standardized tasks

## Installation

1. Ensure you have Rust installed: https://rustup.rs/
2. Clone this repository
3. Build the project: `cargo build --release`
4. Optionally, install the binary globally by linking it: `ln -s target/release/attotool /usr/local/bin/attotool`
5. Set your OpenRouter (or OpenAI, to the same env var) API key: `export OPENROUTER_API_KEY=your_key_here`

## Usage

```bash
# Infinite loop (default)
attotool "read ./url.txt, fetch that url and describe the result as a markdown document"

# Single tool call
attotool --max-tool-calls 1 "curl the ubuntu homepage"

# Specify model and other options
attotool --model "openai/gpt-4" --max-tokens 4000 "your task here" --tool-call-details

# Continue a previous conversation
attotool --continue "your follow-up task here"
```

### CLI Options

- `--model`: LLM model to use (default: mistralai/mistral-small-3.1-24b-instruct)
- `--max-tokens`: Maximum tokens for response (default: 2000)
- `--base-url`: API base URL (default: https://openrouter.ai/api/v1, use https://api.openai.com/v1 for OpenAI)
- `--input`: Task description (can also be provided as the first positional argument)
- `--max-tool-calls`: Maximum number of tool calls (default: 0 for infinite)
- `--retries`: Number of retries for API calls (default: 3)
- `--verbose`: Enable detailed output including raw API responses
- `--tool-call-details`: Show detailed tool call results and execution output
- `--disable-agents-md`: Disable automatic loading of AGENTS.md (default: false)
- `--yolo`: 🚩 Enable YOLO mode (skips approval prompts for destructive operations and removes ask_for_clarification tool)
- `--continue` / `-c`: Reads the existing history.yaml and continues the conversation with a new user message
- `--plan` / `-p`: Enable plan mode (read-only phase, modifications discouraged)

## Recommended Models

The following models have been tested and have worked at least once with attotool:

- z-ai/glm-4-32b
- mistralai/mistral-7b-instruct
- google/gemma-3-27b-it
- openai/gpt-oss-20b
- openai/gpt-4o-mini
- qwen/qwen-2.5-7b-instruct
- qwen/qwen-2.5-72b-instruct
- mistralai/mistral-nemo
- **mistralai/mistral-small-3.1-24b-instruct**
- mistralai/devstral-small-2505
- deepseek/deepseek-chat-v3-0324
- x-ai/grok-code-fast-1
- x-ai/grok-4-fast
- x-ai/grok-3-mini

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
