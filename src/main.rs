use attotool::loop_tools_until_finish;
use clap::Parser;

mod attotool;
mod tools;

#[derive(Parser)]
#[command(name = "attotool")]
struct Args {
    // Known decent models: openai/gpt-oss-20b, qwen/qwen-2.5-7b-instruct, mistralai/mistral-7b-instruct, mistralai/mistral-small-3.1-24b-instruct
    #[arg(long, default_value = "mistralai/mistral-small-3.1-24b-instruct")]
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
    #[arg(long)]
    verbose: bool,
    #[arg(long)]
    tool_call_details: bool,
    #[arg(long)]
    disable_agents_md: bool,
    #[arg(long)]
    yolo: bool,
    #[arg(
        long,
        help = "Reads the existing history.yaml and continues the conversation with a new user message"
    )]
    r#continue: bool,
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
        &args.base_url,
        args.verbose,
        args.tool_call_details,
        args.disable_agents_md,
        args.yolo,
        args.r#continue,
    )
    .await
    .unwrap();
    return;
}
