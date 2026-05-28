import * as vscode from 'vscode';

import * as fs from 'fs';
import * as path from 'path';

export async function activate(context: vscode.ExtensionContext) {
    console.log('Realm Tome Builder: Context activated. Registering MCP Server...');

    // Path to the bundled MCP server
    const mcpServerPath = path.join(context.extensionPath, 'mcp-server', 'dist', 'index.js');

    if (!fs.existsSync(mcpServerPath)) {
        vscode.window.showErrorMessage(`Realm Tome Builder: MCP server not found at ${mcpServerPath}`);
        // We do not return immediately, so the command registration still succeeds
    }

    try {
        await injectClineWorkspaceConfig(mcpServerPath);
        await injectVsCodeMcpWorkspaceConfig(mcpServerPath);
    } catch (e) {
        console.error('Failed to inject config:', e);
    }

    // Command to manually trigger registration if needed
    context.subscriptions.push(
        vscode.commands.registerCommand('tomeBuilder.registerMcp', async () => {
            await injectClineWorkspaceConfig(mcpServerPath);
            await injectVsCodeMcpWorkspaceConfig(mcpServerPath);
            vscode.window.showInformationMessage('Realm Tome Builder: MCP server registered with workspace settings!');
        })
    );

    // Prompt the user once on first activation
    const hasPrompted = context.workspaceState.get('tomeBuilder.hasPromptedMcp', false);
    if (!hasPrompted) {
        vscode.window.showInformationMessage(
            'Realm Tome Builder initialized. The Tome MCP server is now available for Cursor, Cline, and Copilot (once native support lands).',
            'OK'
        );
        context.workspaceState.update('tomeBuilder.hasPromptedMcp', true);
    }
}

export function deactivate() { }

async function injectClineWorkspaceConfig(mcpServerPath: string) {
    if (!vscode.workspace.workspaceFolders) {
        return;
    }

    for (const folder of vscode.workspace.workspaceFolders) {
        const vscodeDir = path.join(folder.uri.fsPath, '.vscode');
        if (!fs.existsSync(vscodeDir)) {
            fs.mkdirSync(vscodeDir, { recursive: true });
        }

        const clineConfigPath = path.join(vscodeDir, 'cline_mcp_settings.json');
        let config: any = { mcpServers: {} };

        if (fs.existsSync(clineConfigPath)) {
            try {
                const content = fs.readFileSync(clineConfigPath, 'utf8');
                if (content.trim()) {
                    config = JSON.parse(content);
                }
            } catch (e) {
                console.error('Error parsing cline config', e);
            }
        }

        if (!config.mcpServers) {
            config.mcpServers = {};
        }

        // Use path.normalize to prevent escaped Windows backslashes ending up weird, though stringify handles it.
        config.mcpServers["tome-builder"] = {
            "command": "node",
            "args": [mcpServerPath.replace(/\\/g, '/')],
            "disabled": false,
            "alwaysAllow": []
        };


        fs.writeFileSync(clineConfigPath, JSON.stringify(config, null, 2));
    }
}

async function injectVsCodeMcpWorkspaceConfig(mcpServerPath: string) {
    if (!vscode.workspace.workspaceFolders) {
        return;
    }

    for (const folder of vscode.workspace.workspaceFolders) {
        const vscodeDir = path.join(folder.uri.fsPath, '.vscode');
        if (!fs.existsSync(vscodeDir)) {
            fs.mkdirSync(vscodeDir, { recursive: true });
        }

        const mcpConfigPath = path.join(vscodeDir, 'mcp.json');
        let config: any = { mcpServers: {} };

        if (fs.existsSync(mcpConfigPath)) {
            try {
                const content = fs.readFileSync(mcpConfigPath, 'utf8');
                if (content.trim()) {
                    config = JSON.parse(content);
                }
            } catch (e) {
                console.error('Error parsing mcp config', e);
            }
        }

        if (!config.mcpServers) {
            config.mcpServers = {};
        }

        // Use path.normalize to prevent escaped Windows backslashes ending up weird
        config.mcpServers["tome-builder"] = {
            "command": "node",
            "args": [mcpServerPath.replace(/\\/g, '/')]
        };

        fs.writeFileSync(mcpConfigPath, JSON.stringify(config, null, 2));

    }
}
