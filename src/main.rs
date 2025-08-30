use clap::Parser;
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
use crate::tools::ShellTool;

const CHAIN_OF_THOUGHT_PROMPT: &str =
    "
You are an assistant that extracts reasoning steps from a given prompt.
Do not return text, only return a tool call.
";

#[derive(Deserialize, Serialize, Debug, Clone, JsonSchema)]
struct ChainOfThoughtSteps {
    steps: Vec<String>,
}

struct ReasoningAgent<M: CompletionModel> {
    chain_of_thought_extractor: Extractor<M, ChainOfThoughtSteps>,
    executor: Agent<M>,
}

impl<M: CompletionModel> Prompt for ReasoningAgent<M> {
    #[allow(refining_impl_trait)]
    async fn prompt(&self, prompt: impl Into<Message> + Send) -> Result<String, PromptError> {
        let prompt: Message = prompt.into();
        println!("prompt: {:?} \n", &prompt);

        let mut chat_history = vec![prompt.clone()];
        let extracted = self.chain_of_thought_extractor.extract(prompt).await.map_err(|e| {
            tracing::error!("Extraction error: {:?}", e);
            CompletionError::ProviderError("".into())
        })?;
        if extracted.steps.is_empty() {
            return Ok("No reasoning steps provided.".into());
        }
        let mut reasoning_prompt = String::new();
        for (i, step) in extracted.steps.iter().enumerate() {
            reasoning_prompt.push_str(&format!("Step {}: {}\n", i + 1, step));
        }
        let response = self.executor
            .prompt(reasoning_prompt.as_str())
            .with_history(&mut chat_history)
            .multi_turn(20).await?;
        tracing::info!(
            "full chat history generated: {}",
            serde_json::to_string_pretty(&chat_history).unwrap()
        );
        Ok(response)
    }
}

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

    // Create agent with a preamble and available tools
    let agent = ai_client
        .agent(anthropic::CLAUDE_4_SONNET)
        .preamble(
            "You are an assistant here to help the user select which tool is most appropriate to perform the task specified by the user.
            Follow these instructions closely.
            1. Consider the user's request carefully and identify the core elements of the request.
            2. Select which tool among those made available to you is appropriate given the context.
            3. This is very important: never perform the operation yourself.          
            "
        )
        .max_tokens(1024)
        .tool(RestApiTool)
        .tool(WebSearch)
        .tool(ShellTool)
        .build();

    // Prompt the agent and print the response using the command line argument
    let result = agent.prompt(&args.prompt).multi_turn(20).await?;

    println!("\n\nReasoning Agent: {result}");

    Ok(())
}
