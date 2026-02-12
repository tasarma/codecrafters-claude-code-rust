use async_openai::{Client, config::OpenAIConfig};
use clap::Parser;
use dotenvy::dotenv;
use serde::Deserialize;
use serde_json::json;
use std::{env, process};

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    #[arg(short = 'p', long)]
    prompt: String,
}

#[derive(Deserialize, Debug)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize, Debug)]
struct Choice {
    message: Message,
}

#[derive(Deserialize, Debug)]
struct Message {
    role: String,
    content: Option<String>,
    tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Deserialize, Debug)]
struct ToolCall {
    id: String,
    #[serde(rename = "type")]
    tool_type: String,
    function: Function,
}

#[derive(Deserialize, Debug)]
struct Function {
    name: String,
    #[serde(rename = "arguments")]
    args: String,
}

#[derive(Deserialize, Debug)]
struct ReadArgs {
    file_path: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let args = Args::parse();

    let base_url = env::var("OPENROUTER_BASE_URL")
        .unwrap_or_else(|_| "https://generativelanguage.googleapis.com/v1beta/openai".to_string());

    let api_key = env::var("OPENROUTER_API_KEY").unwrap_or_else(|_| {
        eprintln!("OPENROUTER_API_KEY is not set");
        process::exit(1);
    });

    let model = env::var("MODEL").unwrap_or_else(|_| {
        eprintln!("MODEL is not set");
        process::exit(1);
    });

    let config = OpenAIConfig::new()
        .with_api_base(base_url)
        .with_api_key(api_key);

    let client = Client::with_config(config);

    let response_value = client
        .chat()
        .create_byot(json!({
            "model": model,
            "messages": [
                {
                    "role": "user",
                    "content": args.prompt
                }
            ],
            "tools":[
                {
                    "type": "function",
                    "function": {
                        "name": "Read",
                        "description": "Read and return the contents of a file",
                        "parameters": {
                            "type": "object",
                            "properties": {
                                "file_path": {
                                    "type": "string",
                                    "description": "The path to the file to read",
                                }
                            },
                            "required": ["file_path"]
                        }
                    }
                }
            ],
        }))
        .await?;

    // You can use print statements as follows for debugging, they'll be visible when running tests.
    // eprintln!("Logs from your program will appear here!");

    let response: ChatResponse = serde_json::from_value(response_value)?;
    if let Some(tc) = &response
        .choices
        .get(0)
        .and_then(|c| c.message.tool_calls.as_ref())
    {
        let first_tool = tc.get(0).unwrap();
        let function_name = &first_tool.function.name;

        if function_name == "Read" {
            let args: ReadArgs = serde_json::from_str(&first_tool.function.args)?;

            let content = tokio::fs::read_to_string(&args.file_path).await.unwrap();
            print!("{}", content);
        }
    } else {
        eprintln!("Unexpected response format: {:#?}", response);
    }

    Ok(())
}
