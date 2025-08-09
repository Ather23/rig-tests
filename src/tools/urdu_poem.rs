use serde::{ Deserialize, Serialize };
use rig::{ tool::Tool, completion::ToolDefinition };
use anyhow::Result;

#[derive(Deserialize, Serialize)]
pub struct UrduPoemTool;

#[derive(Deserialize, Serialize)]
pub struct UrduPoemArgs {
    pub topic: String,
}

#[derive(Debug, thiserror::Error)]
pub enum UrduPoemError {
    #[error("Failed to generate poem: {0}")] GenerationError(String),
    #[error(transparent)] Other(#[from] anyhow::Error),
}

impl Tool for UrduPoemTool {
    const NAME: &'static str = "urdu_poem";
    type Error = UrduPoemError;
    type Args = UrduPoemArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "urdu_poem".to_string(),
            description: "Generates an Urdu poem on a given topic.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "topic": {
                        "type": "string",
                        "description": "The topic for the Urdu poem"
                    }
                },
                "required": ["topic"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Placeholder: In a real implementation, this would call an LLM or a poem API
        let poem = format!(
            "{}\n{}\n{}\n{}\n{}",
            "یہ نظم ایک خودکار نظام نے لکھی ہے",
            format!("موضوع: {}", args.topic),
            "خوابوں کی وادی میں چلتے ہیں ہم",
            "محبت کی خوشبو میں پلتے ہیں ہم",
            "زندگی کے رنگوں میں ڈھلتے ہیں ہم"
        );
        Ok(poem)
    }
}
