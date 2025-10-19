use async_trait::async_trait;
use chrono::Utc;
use clap::{ Parser, ValueEnum };
use rig::agent::AgentBuilder;
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

#[derive(Debug, Clone, ValueEnum)]
enum ModelProvider {
    Anthropic,
    Gemini,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    prompt: String,

    #[arg(short, long, value_enum, default_value_t = ModelProvider::Anthropic)]
    model: ModelProvider,
}

#[async_trait]
trait RunnableAgent: Send + Sync {
    async fn run(&self, prompt: &str, max_turns: usize) -> Result<String, PromptError>;
}

#[async_trait]
impl<M: CompletionModel + Send + Sync> RunnableAgent for Agent<M> {
    async fn run(&self, prompt: &str, max_turns: usize) -> Result<String, PromptError> {
        self.prompt(prompt).multi_turn(max_turns).await
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Create agent using the factory
    let agent = get_agent(args.model);

    // Prompt the agent and print the response using the command line argument
    let result = agent.run(&args.prompt, 20).await?;

    println!("\n\nReasoning Agent: {result}");

    Ok(())
}

fn get_agent(provider: ModelProvider) -> Box<dyn RunnableAgent> {
    let todays_date = chrono::Utc::now().with_timezone(&Toronto);
    let preamble =
        format!(r#"
            # Goal:
            You are an assistant here to help the user select which tool is most appropriate to perform the task specified by the user.
            Follow these instructions closely.
            1. Consider the user's request carefully and identify the core elements of the request.
            2. Select which tool among those made available to you is appropriate given the context.
            3. This is very important: never perform the operation yourself.
            
            # Context: 
            Todays date is: {}"#, todays_date);

    match provider {
        ModelProvider::Anthropic => {
            let client: anthropic::Client = anthropic::Client::from_env();
            let agent = client
                .agent(anthropic::CLAUDE_4_SONNET)
                .preamble(&preamble)
                .max_tokens(1024)
                .tool(RestApiTool)
                .tool(WebSearch)
                .tool(ShellTool)
                .tool(LinkToMarkdown)
                .build();
            Box::new(agent)
        }
        ModelProvider::Gemini => {
            let client: gemini::Client = gemini::Client::from_env();
            let agent = client
                .agent(gemini::completion::GEMINI_1_0_PRO)
                .preamble(&preamble)
                .max_tokens(1024)
                .tool(RestApiTool)
                .tool(WebSearch)
                .tool(ShellTool)
                .tool(LinkToMarkdown)
                .build();
            Box::new(agent)
        }
    }
}
