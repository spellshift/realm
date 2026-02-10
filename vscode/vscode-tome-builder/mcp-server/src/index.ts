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

function findRepoRoot(): string | null {
    let currentDir = __dirname;
    while (currentDir !== path.parse(currentDir).root) {
        if (fs.existsSync(path.join(currentDir, 'go.mod'))) {
            return currentDir;
        }
        if (fs.existsSync(path.join(currentDir, '.git'))) {
             return currentDir;
        }
        currentDir = path.dirname(currentDir);
    }
    return null;
}

const REPO_ROOT = findRepoRoot();

function readDocFile(relativePath: string): string {
    if (!REPO_ROOT) {
        return "Error: Could not locate repository root. Ensure you are running this in the realm repository.";
    }
    const fullPath = path.join(REPO_ROOT, relativePath);
    try {
        return fs.readFileSync(fullPath, 'utf-8');
    } catch (e) {
        return `Error reading file ${fullPath}: ${e}`;
    }
}

function readExample(tomeName: string): { metadata: string, script: string } {
     if (!REPO_ROOT) {
        return {
            metadata: "Error: Could not locate repository root.",
            script: "Error: Could not locate repository root."
        };
    }

    const tomeDir = path.join(REPO_ROOT, `tavern/tomes/${tomeName}`);
    try {
        const metadata = fs.readFileSync(path.join(tomeDir, 'metadata.yml'), 'utf-8');
        const script = fs.readFileSync(path.join(tomeDir, 'main.eldritch'), 'utf-8');
        return { metadata, script };
    } catch (e) {
        return {
            metadata: `Error reading example ${tomeName}: ${e}`,
            script: `Error reading example ${tomeName}: ${e}`
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
        if (topic === "tomes") {
            return {
                content: [{ type: "text", text: readDocFile('docs/_docs/user-guide/tomes.md') }],
            };
        } else if (topic === "eldritch") {
            return {
                content: [{ type: "text", text: readDocFile('docs/_docs/user-guide/eldritch.md') }],
            };
        }
        return { content: [], isError: true };
    }

    if (name === "get_tome_examples") {
        const topic = (args as any).topic;
        if (topic === "file_write") {
            const example = readExample('file_write');
            return {
                content: [
                    { type: "text", text: "## metadata.yml\n" + example.metadata },
                    { type: "text", text: "## main.eldritch\n" + example.script }
                ]
            };
        } else if (topic === "persist_service") {
             const example = readExample('persist_service');
             return {
                content: [
                    { type: "text", text: "## metadata.yml\n" + example.metadata },
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
