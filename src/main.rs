use anyhow::Result;
use rig::completion::Prompt;
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
        .agent(providers::openai::GPT_4)
        .preamble(
            "You are helpful assistant that has access to tools. 
            You are going to help the user answer any questions using the tools provided.
            You can also search the web for help."
        )
        .max_tokens(1024)
        .tool(RestApiTool)
        .tool(WebSearch)
        .tool(ShellTool)
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

    /**
     * 
     *             "You are an assistant here to help the user select which tool is most appropriate to perform arithmetic operations.
            Follow these instructions closely.
            1. Consider the user's request carefully and identify the core elements of the request.
            2. Select which tool among those made available to you is appropriate given the context.
            3. This is very important: never perform the operation yourself.
            "
     * 
     */

    // You are a web search agent that can return results based on users query. You also have shell access if you need it.
    // "Call the REST API at https://jsonplaceholder.typicode.com/todos/1 and show the result."

    let result = rest_api_agent
        .prompt("Clone the langchain repo in D:\\test-rig-agent")
        .multi_turn(20).await?;

    println!("\n\nAgent Response: {result}");

    // let mut stream = rest_api_agent.stream_prompt(
    //     "Your tasks is to clone langchain repo in D:\\test-rig folder.
    //     Create the folder if not there. Try to fix the command if its incorrect"
    // ).await?;

    // stream_to_stdout(&rest_api_agent, &mut stream).await?;

    // if let Some(response) = stream.response {
    //     println!("Usage: {:?} tokens", response.);
    // }

    // println!("Message: {:?}", stream.choice);

    Ok(())
}
