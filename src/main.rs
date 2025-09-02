use chrono::Utc;
use clap::Parser;
use rig::pipeline::new;
use rig::providers::{ anthropic, gemini };
use rig::{ prelude::*, providers };
use rig::{
    agent::Agent,
    completion::{ CompletionError, CompletionModel, Prompt, PromptError, ToolDefinition },
    extractor::Extractor,
    message::Message,
    providers::openai,
    tool::Tool,
};
use schemars::JsonSchema;
use serde::{ Deserialize, Serialize };
use serde_json::json;
use tracing;

mod tools;
use tools::web_search::*;
use tools::rest_api::*;
use crate::tools::link_to_markdown::LinkToMarkdown;
use crate::tools::ShellTool;
use chrono_tz::America::Toronto;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    prompt: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // tracing_subscriber::fmt().with_max_level(tracing::Level::DEBUG).with_target(false).init();

    // Create Anthropic client
    let ai_client: anthropic::Client = anthropic::Client::from_env();
    let todays_date = chrono::Utc::now().with_timezone(&Toronto);

    // Create agent with a preamble and available tools
    let agent = ai_client
        .agent(anthropic::CLAUDE_4_SONNET)
        .preamble(
            &format!(r#"
            # Goal:
            You are an assistant here to help the user select which tool is most appropriate to perform the task specified by the user.
            Follow these instructions closely.
            1. Consider the user's request carefully and identify the core elements of the request.
            2. Select which tool among those made available to you is appropriate given the context.
            3. This is very important: never perform the operation yourself.
            
            # Context: 
            Todays date is: {}"#, todays_date)
        )
        .max_tokens(1024)
        .tool(RestApiTool)
        .tool(WebSearch)
        .tool(ShellTool)
        .tool(LinkToMarkdown)
        .build();

    // Prompt the agent and print the response using the command line argument
    let result = agent.prompt(&args.prompt).multi_turn(20).await?;

    println!("\n\nReasoning Agent: {result}");

    Ok(())
}
