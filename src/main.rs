use attotool::{loop_tools_until_finish, one_function_call};
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
    #[arg(long)]
    r#loop: bool,
    #[arg(long, default_value_t = 3)]
    retries: u32,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let message = args.input.as_ref().unwrap_or(&"".to_string()).clone();
    if args.r#loop {
        loop_tools_until_finish(message, &args.model, args.retries)
            .await
            .unwrap();
    } else {
        one_function_call(message, &args.model, args.retries).await.unwrap();
    }
    return;
}
