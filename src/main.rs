use attotool::loop_tools_until_finish;
use clap::Parser;

mod attotool;
mod tools;

#[derive(Parser)]
#[command(name = "attotool-rs")]
struct Args {
    // Known decent models: openai/gpt-oss-20b, qwen/qwen-2.5-7b-instruct, mistralai/mistral-7b-instruct, z-ai/glm-4-32b
    #[arg(long, default_value = "openai/gpt-oss-20b")]
    model: String,
    #[arg(long, default_value_t = 2000)]
    max_tokens: u32,
    #[arg(long, default_value = "https://openrouter.ai/api/v1")]
    base_url: String,
    #[arg(long)]
    input: Option<String>,
    #[arg(
        long,
        default_value_t = 0,
        help = "Maximum number of tool calls (0 for infinite)"
    )]
    max_tool_calls: u32,
    #[arg(long, default_value_t = 3)]
    retries: u32,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let message = args.input.as_ref().unwrap_or(&"".to_string()).clone();
    loop_tools_until_finish(
        message,
        &args.model,
        args.retries,
        args.max_tokens,
        args.max_tool_calls,
    )
    .await
    .unwrap();
    return;
}
