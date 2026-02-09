use async_openai::{Client, config::OpenAIConfig};
use clap::Parser;
use dotenvy::dotenv;
use serde_json::{Value, json};
use std::{env, process};

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    #[arg(short = 'p', long)]
    prompt: String,
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

    #[allow(unused_variables)]
    let response: Value = client
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

    // TODO: Uncomment the lines below to pass the first stage
    if let Some(content) = response["choices"][0]["message"]["content"].as_str() {
        println!("{}", content);
    } else {
        eprintln!("Unexpected response format: {:?}", response);
    }

    Ok(())
}
