use anyhow::Result;
use rig::prelude::*;
use rig::streaming::stream_to_stdout;
use rig::{ completion::ToolDefinition, providers, streaming::StreamingPrompt, tool::Tool };
use serde::{ Deserialize, Serialize };

mod tools;
use tools::web_search::*;

use crate::tools::{ ShellTool, UrduPoemTool };
#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt().init();

    // Create agent with a single context prompt and two tools
    let search_agent = providers::gemini::Client
        ::from_env()
        .agent(providers::gemini::completion::GEMINI_2_0_FLASH)
        .preamble(r#"You are an urdu poet."#)
        .max_tokens(1024)
        .tool(WebSearch)
        .tool(ShellTool)
        .tool(UrduPoemTool)
        .build();

    // You are an agent that has access to PowerShell.
    // You also have access to the web if you want to look up documentation.
    // Make sure you respond in a nice and friendly way.

    // You are a web search agent that can return results based on users query. You also have shell access if you need it.

    let mut stream = search_agent.stream_prompt(
        "Write something in the same prose as jaun elia"
    ).await?;

    stream_to_stdout(&search_agent, &mut stream).await?;

    if let Some(response) = stream.response {
        println!("Usage: {:?} tokens", response.usage_metadata.total_token_count);
    }

    println!("Message: {:?}", stream.choice);

    Ok(())
}
