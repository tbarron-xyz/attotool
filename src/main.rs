use attotool::loop_tools_until_finish;
use clap::Parser;
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::Path;

mod attotool;
mod tools;
mod yaml_parsing;

#[derive(Parser)]
#[command(name = "attotool")]
struct Args {
    // Known decent models: openai/gpt-oss-20b, qwen/qwen-2.5-7b-instruct, mistralai/mistral-7b-instruct, mistralai/mistral-small-3.1-24b-instruct
    #[arg(long)]
    model: Option<String>,
    #[arg(long, default_value_t = 2000)]
    max_tokens: u32,
    #[arg(long, default_value = "https://openrouter.ai/api/v1")]
    base_url: String,
    #[arg(long)]
    input: Option<String>,
    #[arg(index = 1, conflicts_with = "input")]
    positional_input: Option<String>,
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
        short = 'c',
        long,
        help = "Reads the existing history.yaml and continues the conversation with a new user message"
    )]
    r#continue: bool,
    #[arg(long, short = 'p')]
    plan: bool,
}

#[derive(Deserialize)]
struct Config {
    model: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let message =
        args.input.or(args.positional_input).unwrap_or("".to_string()).clone();

    let config_path = format!(
        "{}/.config/attotool.yaml",
        env::var("HOME").expect("HOME not set")
    );
    let config_model = {
        let mut cm = "mistralai/mistral-small-3.1-24b-instruct".to_string();
        if Path::new(&config_path).exists() {
            if let Ok(content) = fs::read_to_string(&config_path) {
                if let Ok(config) = serde_yaml::from_str::<Config>(&content) {
                    if let Some(m) = config.model {
                        cm = m;
                    }
                }
            }
        }
        cm
    };
    let model = args.model.as_ref().unwrap_or(&config_model);

    loop_tools_until_finish(
        message,
        &model,
        args.retries,
        args.max_tokens,
        args.max_tool_calls,
        &args.base_url,
        args.verbose,
        args.tool_call_details,
        args.disable_agents_md,
        args.yolo,
        args.r#continue,
        args.plan,
    )
    .await
    .unwrap();
}
