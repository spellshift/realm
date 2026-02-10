# Realm Tome Builder VS Code Extension

**Tome Builder** is an AI-powered VS Code extension designed to assist operators and developers in creating **Realm Tomes**. It leverages the **Gemini API** and a local **Model Context Protocol (MCP)** server to provide expert guidance, documentation, and syntax validation for the **Eldritch DSL**.

## Features

*   **AI Chat Assistant**: Interact with an "Expert Tome Developer" persona to generate Tome logic and configuration.
*   **Context-Aware**: The AI has access to up-to-date Eldritch documentation and real-world Tome examples via the embedded MCP server.
*   **One-Click Save**: Easily save generated `metadata.yml` and `main.eldritch` code blocks directly to your workspace.
*   **Model Selection**: Dynamically list and select available Gemini models (e.g., `gemini-2.0-flash-exp`) compatible with your API key.
*   **Syntax Validation**: The AI validates generated code structure before presenting it to you.

## Architecture

This extension utilizes a **Model Context Protocol (MCP)** architecture:

1.  **VS Code Extension**: Provides the Chat UI and manages the lifecycle of the MCP server.
2.  **MCP Server** (Local): A Node.js server running locally that exposes:
    *   `get_documentation`: Eldritch and Tome reference materials.
    *   `get_tome_examples`: Best-practice examples (e.g., file writing, service persistence).
    *   `validate_tome_structure`: Basic structural validation for Tomes.
3.  **LLM (Gemini)**: The extension connects to Google's Gemini API, using the MCP client to call tools on the local server.

## Installation & Setup

### Prerequisites

*   Node.js (v18+)
*   npm

### Building from Source

1.  **Clone the repository** and navigate to the extension folder:
    ```bash
    cd vscode-tome-builder
    ```

2.  **Install Dependencies**:
    You need to install dependencies for both the extension and the internal MCP server.
    ```bash
    # Install extension dependencies
    npm install

    # Install MCP server dependencies and build it
    cd mcp-server
    npm install
    npm run build
    cd ..
    ```

3.  **Compile the Extension**:
    ```bash
    npm run compile
    ```

### Configuration

Before using the extension, you must provide your Google Gemini API Key.

1.  Open VS Code Settings (`Ctrl+,` or `Cmd+,`).
2.  Search for **Tome Builder**.
3.  Enter your key in **Tome Builder > Llm: Api Key**.
4.  (Optional) You can select a default model in **Tome Builder > Llm: Model**, or use the dropdown in the chat interface.

## Usage

1.  Open the **Tome Builder** view from the Activity Bar (beaker icon).
2.  Type a request in the chat, for example:
    > "Create a tome that installs a systemd service to run /usr/bin/myimplant."
3.  The AI will analyze your request, consult documentation/examples if needed, and generate the required files.
4.  Click the **Save** button above the code blocks to save `metadata.yml` and `main.eldritch` to your current workspace.

## Troubleshooting

*   **"MCP Client not connected"**: Ensure you ran `npm run build` in the `mcp-server` directory. The extension relies on `mcp-server/dist/index.js` existing.
*   **404 Model Not Found**: The configured model might not be available in your region or for your API key tier. Use the dropdown in the chat header to select a valid model.
