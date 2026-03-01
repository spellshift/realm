import * as vscode from 'vscode';
import { ChatPanel } from './panels/ChatPanel';
import { McpClientService } from './services/McpClientService';

let mcpService: McpClientService;

export async function activate(context: vscode.ExtensionContext) {
    console.log('Tome Builder extension is now active!');

    // Start MCP Service
    mcpService = new McpClientService(context);
    try {
        await mcpService.start();
    } catch (e) {
        vscode.window.showErrorMessage(`Failed to start MCP Server: ${e}`);
    }

    // Register View
    const provider = new ChatPanel(context.extensionUri, context, mcpService);

    context.subscriptions.push(
        vscode.window.registerWebviewViewProvider(ChatPanel.viewType, provider)
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('tomeBuilder.openChat', () => {
             // Focus the view
             vscode.commands.executeCommand('workbench.view.extension.tome-builder-view');
        })
    );
}

export async function deactivate() {
    if (mcpService) {
        await mcpService.stop();
    }
}
