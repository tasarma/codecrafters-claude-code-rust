# Rust AI Coding Assistant

A CLI-based AI assistant built in Rust that interacts with Large Language Models (LLMs) to perform tasks using tool calls. This project demonstrates how to build an OpenAI-API-compatible agent from scratch, featuring custom tool integration and an autonomous execution loop.

## Features

-   **Autonomous Agent Loop**: Implements a feedback loop where the AI can think, call tools, receive outputs, and continue until the task is complete.
-   **Tool Integration**: Features a custom `Read` tool allowing the agent to access and read files from the local filesystem.
-   **OpenAI-Compatible Client**: built using `async-openai`, compatible with any provider supporting the OpenAI API spec (e.g., OpenRouter, Google Gemini, OpenAI).
-   **Robust CLI**: precise command-line argument parsing using `clap`.
-   **State Management**: maintain conversation history and context across multiple turns.

## Tech Stack

-   **Language**: Rust
-   **Async Runtime**: Tokio
-   **API Client**: async-openai
-   **CLI Parsing**: Clap
-   **Serialization**: Serde & Serde JSON
-   **Configuration**: Dotenvy

## Getting Started

### Prerequisites

-   Rust and Cargo installed on your machine.

### Installation

1.  Clone the repository:
    ```bash
    git clone https://github.com/tasarma/codecrafters-claude-code-rust
    cd codecrafters-claude-code-rust
    ```

2.  Set up your environment variables. Create a `.env` file in the project root:
    ```env
    # Required: Your API Key
    OPENROUTER_API_KEY=your_api_key_here

    # Required: The model you want to use
    MODEL=gpt-4o

    # Optional: Base URL (Defaults to Google's OpenAI-compatible endpoint if omitted)
    # OPENROUTER_BASE_URL=https://openrouter.ai/api/v1
    ```

3.  Build the project:
    ```bash
    cargo build --release
    ```

## Usage

Run the assistant by providing a prompt with the `-p` (or `--prompt`) flag:

```bash
cargo run -- -p "Read src/main.rs and explain how the tool use loop works."
```
**Output**
```
The tool use loop in `src/main.rs` implements a standard ReAct (Reasoning and Acting) pattern. Here is a step-by-step breakdown of how it works:

### 1. Initialization
The loop starts with a conversation history (`messages`) containing only the user's prompt and a definition of available `tools` (in this case, the `Read` tool).

### 2. The Loop Logic
The program enters an infinite `loop` that follows these steps:

*   **Request to LLM**: It sends the current `messages` history and the `tools` definition to the model.
*   **Update History**: When the LLM responds, the program immediately adds the LLM's message to the `messages` vector. This is important because the API requires the assistant's "intent" to call a tool to be part of the conversation history.
*   **Branching Logic**: The program checks if the assistant's message contains `tool_calls`:

#### A. If Tool Calls are present:
1.  **Execution**: It iterates through each tool call requested by the model. 
2.  **Logic Dispatch**: It looks for the tool name (e.g., `"Read"`). If matched, it parses the arguments and executes the actual Rust code (using `tokio::fs::read_to_string`).
3.  **Result Integration**: The output of the tool (the file content or an error) is pushed back into the `messages` history as a new message with the role `"tool"`. It includes a `tool_call_id` so the LLM knows which request this result belongs to.
4.  **Recurse**: The loop **continues**. It goes back to the top and sends the updated history (which now includes the tool's output) back to the LLM. This allows the LLM to "see" the data it requested and decide what to do next.

#### B. If No Tool Calls are present (Terminal State):
1.  **Output**: This means the LLM has finished its task and is providing a final text response.
2.  **Termination**: The program prints the assistant's content to the console and `break`s the loop.

### Summary
The loop allows for **multi-turn reasoning**. The model can call a tool, see the result, and then decide to call another tool (or the same tool with different arguments) multiple times before finally providing an answer to the user.%      
```

### Example Workflow
1.  **User**: Submits a prompt to read a file.
2.  **Agent**: Analyzes the request and decides to call the `Read` tool with the specified path.
3.  **System**: Executes the file read operation and returns the content to the agent.
4.  **Agent**: Reads the content and formulates a response explaining the code.
5.  **User**: Receives the explanation.

## Project Structure

-   `src/main.rs`: The core logic. Initializes the client, defines the `Read` tool schema, and runs the main conversation loop.
-   `.env`: Configuration for API keys and endpoints (excluded from version control).