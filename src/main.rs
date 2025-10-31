use attotool::loop_tools_until_finish;
use clap::Parser;

mod app;
mod attotool;
mod event;
mod output;
mod tool_handler;
mod tools;
mod tui;
mod yaml_utilities;

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
    #[arg(long)]
    ui: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let config_model = yaml_utilities::get_default_model();
    let model = args.model.as_ref().unwrap_or(&config_model).clone();

    if args.ui {
        let mut app = app::App::new(
            model,
            args.retries,
            args.max_tokens,
            args.max_tool_calls,
            args.base_url,
            args.verbose,
            args.tool_call_details,
            args.disable_agents_md,
            args.yolo,
            args.r#continue,
            args.plan,
            true, // ui_mode
        );
        app.run().await.unwrap();
    } else {
        let message = args
            .input
            .or(args.positional_input)
            .unwrap_or("".to_string())
            .clone();

        let mut output = crate::output::StdoutOutput {
            tool_call_details: args.tool_call_details,
        };
        let mut tool_calls = Vec::new();

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
            false, // ui_mode
            None,  // approval_override
            &mut output,
            &mut tool_calls,
        )
        .await
        .unwrap();
    }
}
