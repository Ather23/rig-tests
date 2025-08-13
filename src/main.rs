use anyhow::Result;
use rig::prelude::*;
use rig::streaming::stream_to_stdout;
use rig::{ completion::ToolDefinition, providers, streaming::StreamingPrompt, tool::Tool };
use serde::{ Deserialize, Serialize };

mod tools;
use tools::web_search::*;
use tools::rest_api::*;
use crate::tools::{ ShellTool };
#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt().init();

    let openai_client = providers::openai::Client::from_env();

    let rest_api_agent = openai_client
        .agent(providers::openai::GPT_4O_MINI)
        .preamble("You are an agent that can call RESTful APIs using reqwest.")
        .max_tokens(1024)
        .tool(RestApiTool)
        .build();

    // Create agent with a single context prompt and two tools
    // let search_agent = providers::gemini::Client
    //     ::from_env()
    //     .agent(providers::gemini::completion::GEMINI_2_0_FLASH)
    //     .preamble(r#"You are an urdu poet."#)
    //     .max_tokens(1024)
    //     .tool(WebSearch)
    //     .tool(ShellTool)
    //     .tool(UrduPoemTool)
    //     .build();

    // You are an agent that has access to PowerShell.
    // You also have access to the web if you want to look up documentation.
    // Make sure you respond in a nice and friendly way.

    // You are a web search agent that can return results based on users query. You also have shell access if you need it.

    let mut stream = rest_api_agent.stream_prompt(
        "Call the REST API at https://jsonplaceholder.typicode.com/todos/1 and show the result."
    ).await?;

    stream_to_stdout(&rest_api_agent, &mut stream).await?;

    // if let Some(response) = stream.response {
    //     println!("Usage: {:?} tokens", response.);
    // }

    // println!("Message: {:?}", stream.choice);

    Ok(())
}
