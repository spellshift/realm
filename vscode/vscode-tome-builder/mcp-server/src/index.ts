import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import {
    CallToolRequestSchema,
    ListToolsRequestSchema,
    Tool,
} from "@modelcontextprotocol/sdk/types.js";
import { z } from "zod";
import * as fs from 'fs';
import * as path from 'path';

// --- Helper Functions for Runtime File Reading ---

const GITHUB_RAW_BASE = "https://raw.githubusercontent.com/spellshift/realm/main";

async function readDocFile(relativePath: string): Promise<string> {
    const url = `${GITHUB_RAW_BASE}/${relativePath}`;
    try {
        const response = await fetch(url);
        if (!response.ok) {
            return `Error fetching documentation from GitHub: HTTP ${response.status}`;
        }
        return await response.text();
    } catch (e) {
        return `Error fetching documentation: ${e}`;
    }
}

async function readExample(tomeName: string): Promise<{ metadata: string, script: string }> {
    const tomeDirUrl = `${GITHUB_RAW_BASE}/tavern/tomes/${tomeName}`;
    try {
        const [metaRes, scriptRes] = await Promise.all([
            fetch(`${tomeDirUrl}/metadata.yml`),
            fetch(`${tomeDirUrl}/main.eldritch`)
        ]);

        if (!metaRes.ok || !scriptRes.ok) {
            return {
                metadata: `Error fetching metadata.yml: HTTP ${metaRes.status}`,
                script: `Error fetching main.eldritch: HTTP ${scriptRes.status}`
            };
        }

        const metadata = await metaRes.text();
        const script = await scriptRes.text();

        return { metadata, script };
    } catch (e) {
        return {
            metadata: `Network error fetching metadata: ${e}`,
            script: `Network error fetching script: ${e}`
        };
    }
}

// --- Server Setup ---

const server = new Server(
    {
        name: "tome-builder",
        version: "1.0.0",
    },
    {
        capabilities: {
            tools: {},
        },
    }
);

// Define Tools
const getDocumentationTool: Tool = {
    name: "get_documentation",
    description: "Get documentation for Tomes or Eldritch language.",
    inputSchema: {
        type: "object",
        properties: {
            topic: {
                type: "string",
                enum: ["tomes", "eldritch"],
                description: "The topic to retrieve documentation for."
            }
        },
        required: ["topic"]
    }
};

const getTomeExamplesTool: Tool = {
    name: "get_tome_examples",
    description: "Get example Tomes to understand structure and best practices. Returns metadata and script content.",
    inputSchema: {
        type: "object",
        properties: {
            topic: {
                type: "string",
                enum: ["file_write", "persist_service"],
                description: "The example to retrieve. 'file_write' is simple, 'persist_service' is complex (templates, os checks)."
            }
        },
        required: ["topic"]
    }
};

const validateTomeStructureTool: Tool = {
    name: "validate_tome_structure",
    description: "Validate the structure of a Tome (metadata.yml and main.eldritch).",
    inputSchema: {
        type: "object",
        properties: {
            yaml_content: { type: "string" },
            script_content: { type: "string" }
        },
        required: ["yaml_content", "script_content"]
    }
};

server.setRequestHandler(ListToolsRequestSchema, async () => {
    return {
        tools: [getDocumentationTool, getTomeExamplesTool, validateTomeStructureTool]
    };
});

server.setRequestHandler(CallToolRequestSchema, async (request) => {
    const { name, arguments: args } = request.params;

    if (name === "get_documentation") {
        const topic = (args as any).topic;

        const SYSTEM_PROMPTS = `
        === CRITICAL AI ELDRITCH PROMPT INSTRUCTIONS ===
        1. ALWAYS use \`eprint(...)\` for logging errors to stderr. NEVER use \`print("error: ...")\`.
        2. \`print(...)\` is STRICTLY reserved for actual stdout data output.
        3. Do NOT hallucinate Python standard libraries (e.g. no \`os\`, \`subprocess\`, \`json\`). You MUST use the Eldritch APIs defined below.
        ================================================
        \n\n`;

        if (topic === "tomes") {
            const content = await readDocFile('docs/_docs/user-guide/tomes.md');
            return {
                content: [{ type: "text", text: SYSTEM_PROMPTS + content }],
            };
        } else if (topic === "eldritch") {
            const content = await readDocFile('docs/_docs/user-guide/eldritch.md');
            return {
                content: [{ type: "text", text: SYSTEM_PROMPTS + content }],
            };
        }
        return { content: [], isError: true };
    }

    if (name === "get_tome_examples") {
        const topic = (args as any).topic;
        const SYSTEM_PROMPTS = `
        === CRITICAL AI ELDRITCH PROMPT INSTRUCTIONS ===
        1. ALWAYS use \`eprint(...)\` for logging errors to stderr. NEVER use \`print("error: ...")\`.
        2. \`print(...)\` is STRICTLY reserved for actual stdout data output.
        3. Do NOT hallucinate Python standard libraries (e.g. no \`os\`, \`subprocess\`, \`json\`). You MUST use the Eldritch APIs defined below.
        ================================================
        \n\n`;

        if (topic === "file_write") {
            const example = await readExample('file_write');
            return {
                content: [
                    { type: "text", text: SYSTEM_PROMPTS + "## metadata.yml\n" + example.metadata },
                    { type: "text", text: "## main.eldritch\n" + example.script }
                ]
            };
        } else if (topic === "persist_service") {
            const example = await readExample('persist_service');
            return {
                content: [
                    { type: "text", text: SYSTEM_PROMPTS + "## metadata.yml\n" + example.metadata },
                    { type: "text", text: "## main.eldritch\n" + example.script }
                ]
            };
        }
        return { content: [], isError: true };
    }

    if (name === "validate_tome_structure") {
        const yaml_content = (args as any).yaml_content;
        const script_content = (args as any).script_content;

        const errors: string[] = [];

        if (!yaml_content.includes("name:")) errors.push("metadata.yml missing 'name'");
        if (!yaml_content.includes("description:")) errors.push("metadata.yml missing 'description'");
        if (!yaml_content.includes("tactic:")) errors.push("metadata.yml missing 'tactic'");

        if (!script_content.includes("def main")) errors.push("main.eldritch missing 'def main' function");

        if (errors.length > 0) {
            return {
                content: [{ type: "text", text: "Validation Failed:\n" + errors.join("\n") }],
                isError: true
            };
        }

        return {
            content: [{ type: "text", text: "Validation Passed! The structure looks correct." }]
        };
    }

    throw new Error(`Tool not found: ${name}`);
});

async function main() {
    const transport = new StdioServerTransport();
    await server.connect(transport);
    console.error("Tome MCP Server running on stdio");
}

main().catch((error) => {
    console.error("Fatal error in main():", error);
    process.exit(1);
});
