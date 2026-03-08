# Realm Tome Builder VS Code Extension

<<<<<<< HEAD
**Tome Builder** is a VS Code extension designed to seamlessly integrate the **Realm Tome Model Context Protocol (MCP) Server** with your preferred AI Chat Assistants (like Cline, Continue, and Copilot).

By installing this extension, you enable your AI tools to understand the **Eldritch DSL** and access real-world **Tome** examples, allowing the AI to generate accurate, syntactically correct Tomes for your offensive security workflows.

## Features

*   **Zero-Config Integration**: Automatically registers the bundled Tome MCP Server with popular AI extensions like Cline.
*   **Context-Aware AI**: Provides your AI assistants with up-to-date Eldritch documentation and reference materials.
*   **Copilot Ready**: Exposes the standard VS Code `mcpServers` contribution point for native GitHub Copilot Chat integration (once native MCP support is fully released).
=======
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
>>>>>>> 08809fe2782ce8d0f927507c6c4c63a116c19522

## Installation & Setup

### Prerequisites

*   Node.js (v18+)
*   npm
<<<<<<< HEAD
*   VSCE (Visual Studio Code Extension Manager) - `npm install -g @vscode/vsce`

### Installation via VSIX (Recommended)

To install the extension directly into VS Code, you can package it into a `.vsix` file:
=======

### Building from Source
>>>>>>> 08809fe2782ce8d0f927507c6c4c63a116c19522

1.  **Clone the repository** and navigate to the extension folder:
    ```bash
    cd vscode-tome-builder
    ```

<<<<<<< HEAD
2.  **Install Dependencies & Build the MCP Server**:
    ```bash
    npm install
    npm run build-mcp
    ```

3.  **Package the Extension**:
    ```bash
    vsce package
    ```
    *This will generate a `tome-builder-0.1.0.vsix` file in the directory.*

4.  **Install the Extension**:
    *   Open VS Code.
    *   Go to the Extensions view (`Ctrl+Shift+X` or `Cmd+Shift+X`).
    *   Click the **...** (Views and More Actions) menu in the top right of the Extensions view.
    *   Select **Install from VSIX...**
    *   Locate and select the `tome-builder-0.1.0.vsix` file you just generated.

### Building from Source (Development)

1.  **Clone the repository** and navigate to the extension folder:
    ```bash
    cd vscode-tome-builder
    ```

2.  **Install Dependencies & Build**:
    ```bash
    npm install
    npm run build-mcp
    npm run compile
    ```

### Usage

Once the extension is installed and activated, it will automatically attempt to configure the workspace for supported AI tools.

**For Cline:**
The extension will inject the `tome-builder` MCP server into your workspace's `.vscode/cline_mcp_settings.json`. You can then open Cline and ask it to "Create a Realm Tome to establish persistence".

**For GitHub Copilot Chat (Requires VS Code Insiders):**
Native MCP support in GitHub Copilot is currently in preview and requires using the VS Code Insiders build.
1. Download and install [VS Code Insiders](https://code.visualstudio.com/insiders/).
2. Open your VS Code Settings (`Ctrl+,` or `Cmd+,`).
3. Search for `chat.experimental.mcp` and check the box to enable it.
4. Reload the window. 
5. The extension contributes the `tome-builder` MCP server automatically. You can now tag Copilot in chat (or use inline chat) to ask about Eldritch and Tome creation, and it will route requests to the server.

### Manual Registration
If your AI assistant's settings were not updated automatically, you can manually trigger the registration:
1. Open the Command Palette (`Ctrl+Shift+P` / `Cmd+Shift+P`).
2. Run **Tome Builder: Register MCP Server for AI extensions**.
=======
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
>>>>>>> 08809fe2782ce8d0f927507c6c4c63a116c19522
