import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { StdioClientTransport } from "@modelcontextprotocol/sdk/client/stdio.js";
import * as vscode from 'vscode';
import * as path from 'path';

export class McpClientService {
    private client: Client | undefined;
    private transport: StdioClientTransport | undefined;

    constructor(private context: vscode.ExtensionContext) {}

    async start() {
        const serverPath = path.join(this.context.extensionUri.fsPath, 'mcp-server', 'dist', 'index.js');

        console.log(`Starting MCP server at: ${serverPath}`);

        this.transport = new StdioClientTransport({
            command: "node",
            args: [serverPath]
        });

        this.client = new Client(
            {
                name: "tome-builder-client",
                version: "1.0.0",
            },
            {
                capabilities: {},
            }
        );

        try {
            await this.client.connect(this.transport);
            console.log("MCP Client connected to server");
        } catch (e) {
            console.error("Failed to connect to MCP server:", e);
            vscode.window.showErrorMessage(`Failed to connect to MCP Server: ${e}`);
        }
    }

    async stop() {
        await this.transport?.close();
    }

    getClient(): Client | undefined {
        return this.client;
    }
}
