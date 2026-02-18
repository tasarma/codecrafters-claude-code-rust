use async_openai::{Client, config::OpenAIConfig};
use clap::Parser;
use dotenvy::dotenv;
use serde::Deserialize;
use serde_json::{Value, json};
use std::{env, process};

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    #[arg(short = 'p', long)]
    prompt: String,
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

    // 1. Initialize the conversation history
    let mut messages = vec![json!({
        "role": "user",
        "content": args.prompt
    })];

    let tools = json!([
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
    ]);

    // 2. Enter the Loop
    loop {
        let response: Value = client
            .chat()
            .create_byot(json!({
                "model": model,
                "messages": messages,
                "tools": tools,
            }))
            .await?;

        // You can use print statements as follows for debugging, they'll be visible when running tests.
        // eprintln!("Logs from your program will appear here!");

        // Extract the assistant's message
        let choice = &response["choices"][0];
        let assistant_message = &choice["message"];

        // 3. Record the assistant's response to history
        messages.push(assistant_message.clone());

        // 4. Check for tool calls
        if let Some(tool_calls) = assistant_message["tool_calls"].as_array() {
            for tc in tool_calls {
                let call_id = tc["id"].as_str().unwrap();
                let function_name = tc["function"]["name"].as_str().unwrap();
                let args_str = tc["function"]["arguments"].as_str().unwrap();

                if function_name == "Read" {
                    let read_args: ReadArgs = serde_json::from_str(&args_str)?;

                    // Execute the tool
                    match tokio::fs::read_to_string(&read_args.file_path).await {
                        Ok(content) => {
                            // Add tool result to history
                            messages.push(json!({
                                "role": "tool",
                                "tool_call_id": call_id,
                                "content": content
                            }));
                        }
                        Err(e) => {
                            messages.push(json!({
                                "role": "tool",
                                "tool_call_id": call_id,
                                "content": format!("Error reading file: {}", e)
                            }));
                        }
                    };
                }
            }
        // After handling all tool calls, the loop continues to send the new history back to the LLM
        } else {
            // 5. Repeat until complete: No tool calls means we have a final answer
            if let Some(content) = assistant_message["content"].as_str() {
                print!("{}", content);
            }
            break;
        }
    }

    Ok(())
}
