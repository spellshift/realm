import { GoogleGenerativeAI } from "@google/generative-ai";
import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import * as vscode from 'vscode';

export class LlmService {
    private genAI: GoogleGenerativeAI;
    private model: any;
    private chatSession: any;

    constructor(private apiKey: string, private mcpClient: Client, private modelName: string = "gemini-2.0-flash-exp") {
        this.genAI = new GoogleGenerativeAI(apiKey);
        // Note: Gemini 2.0 might require specific beta API version or model string.
        // Using standard getGenerativeModel.
        this.model = this.genAI.getGenerativeModel({
            model: modelName,
            systemInstruction: "You are an expert Realm Tome developer. Your goal is to help users write 'Tomes' (Eldritch packages). " +
                "You have access to tools to retrieve documentation about Tomes and the Eldritch language. " +
                "ALWAYS check the documentation using 'get_documentation' if you are unsure about syntax or APIs. " +
                "When asked to create a tome, generate the 'metadata.yml' and 'main.eldritch' files. " +
                "Use the 'validate_tome_structure' tool to check your generated code if possible. " +
                "Output the code in markdown blocks (```yaml and ```python or ```eldritch). " +
                "For 'main.eldritch', use python syntax highlighting but remember it is Starlark-based Eldritch."
        });
    }

    setModel(modelName: string) {
        this.modelName = modelName;
        this.model = this.genAI.getGenerativeModel({
            model: modelName,
            systemInstruction: "You are an expert Realm Tome developer. Your goal is to help users write 'Tomes' (Eldritch packages). " +
                "You have access to tools to retrieve documentation about Tomes and the Eldritch language. " +
                "ALWAYS check the documentation using 'get_documentation' if you are unsure about syntax or APIs. " +
                "When asked to create a tome, generate the 'metadata.yml' and 'main.eldritch' files. " +
                "Use the 'validate_tome_structure' tool to check your generated code if possible. " +
                "Output the code in markdown blocks (```yaml and ```python or ```eldritch). " +
                "For 'main.eldritch', use python syntax highlighting but remember it is Starlark-based Eldritch."
        });
        this.chatSession = undefined; // Reset session
    }

    async listAvailableModels(): Promise<string[]> {
        try {
            // The SDK doesn't export listModels directly on GoogleGenerativeAI, so we fetch manually.
            // Documentation: https://ai.google.dev/api/rest/v1beta/models/list
            const response = await fetch(`https://generativelanguage.googleapis.com/v1beta/models?key=${this.apiKey}`);
            if (!response.ok) {
                console.error(`Failed to list models: ${response.statusText}`);
                return ["gemini-2.0-flash-exp", "gemini-1.5-pro", "gemini-1.5-flash"];
            }
            const data = await response.json();
            if (data && data.models) {
                return data.models
                    .map((m: any) => m.name.replace('models/', ''))
                    .filter((name: string) => name.includes('gemini'));
            }
        } catch (e) {
            console.error("Error listing models:", e);
        }
        return ["gemini-2.0-flash-exp", "gemini-1.5-pro", "gemini-1.5-flash"];
    }

    async startSession() {
        if (!this.mcpClient) throw new Error("MCP Client not initialized");

        console.log("Listing tools from MCP...");
        const toolsList = await this.mcpClient.listTools();
        console.log(`Found ${toolsList.tools.length} tools.`);

        const tools = toolsList.tools.map((tool: any) => {
            // Ensure inputSchema is clean for Gemini
            return {
                name: tool.name,
                description: tool.description,
                parameters: tool.inputSchema
            };
        });

        this.chatSession = this.model.startChat({
            tools: [{ functionDeclarations: tools }]
        });
        console.log("Gemini Chat Session started.");
    }

    async sendMessage(message: string): Promise<string> {
        if (!this.chatSession) await this.startSession();

        console.log(`Sending message to Gemini: ${message}`);
        let result = await this.chatSession.sendMessage(message);
        let response = await result.response;

        let functionCalls = response.functionCalls();

        while (functionCalls && functionCalls.length > 0) {
            console.log("Model requested function calls:", functionCalls);
            const functionResponses = [];

            for (const call of functionCalls) {
                console.log(`Executing tool: ${call.name}`);

                try {
                    const mcpResult = await this.mcpClient.callTool({
                        name: call.name,
                        arguments: call.args
                    });

                    // MCP returns content: [{ type: 'text', text: '...' }]
                    // We join them for the model
                    // @ts-ignore
                    const textContent = mcpResult.content.map((c: any) => c.text).join("\n");

                    console.log(`Tool ${call.name} executed successfully.`);

                    functionResponses.push({
                        functionResponse: {
                            name: call.name,
                            response: { content: textContent }
                        }
                    });
                } catch (e) {
                    console.error(`Error executing tool ${call.name}:`, e);
                    functionResponses.push({
                        functionResponse: {
                            name: call.name,
                            response: { error: String(e) }
                        }
                    });
                }
            }

            console.log("Sending function responses back to Gemini...");
            result = await this.chatSession.sendMessage(functionResponses);
            response = await result.response;
            functionCalls = response.functionCalls();
        }

        return response.text();
    }
}
