# Realm Tome Builder VS Code Extension

**Tome Builder** is a VS Code extension designed to seamlessly integrate the **Realm Tome Model Context Protocol (MCP) Server** with your preferred AI Chat Assistants (like Cline, Continue, and Copilot).

By installing this extension, you enable your AI tools to understand the **Eldritch DSL** and access real-world **Tome** examples, allowing the AI to generate accurate, syntactically correct Tomes for your offensive security workflows.

## Features

*   **Zero-Config Integration**: Automatically registers the bundled Tome MCP Server with popular AI extensions like Cline.
*   **Context-Aware AI**: Provides your AI assistants with up-to-date Eldritch documentation and reference materials.
*   **Copilot Ready**: Exposes the standard VS Code `mcpServers` contribution point for native GitHub Copilot Chat integration (once native MCP support is fully released).

## Installation & Setup

### Prerequisites

*   Node.js (v18+)
*   npm
*   VSCE (Visual Studio Code Extension Manager) - `npm install -g @vscode/vsce`

### Installation via VSIX (Recommended)

To install the extension directly into VS Code, you can package it into a `.vsix` file:

1.  **Clone the repository** and navigate to the extension folder:
    ```bash
    cd vscode-tome-builder
    ```

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
