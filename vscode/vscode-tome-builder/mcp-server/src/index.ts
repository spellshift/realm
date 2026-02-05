import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import {
  CallToolRequestSchema,
  ListToolsRequestSchema,
  Tool,
} from "@modelcontextprotocol/sdk/types.js";
import { z } from "zod";
import { getTomesDoc, getEldritchDoc } from "./docs";
import { getFileWriteExample, getPersistServiceExample } from "./examples";

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
                content: [{ type: "text", text: getTomesDoc() }],
            };
        } else if (topic === "eldritch") {
            return {
                content: [{ type: "text", text: getEldritchDoc() }],
            };
        }
        return { content: [], isError: true };
    }

    if (name === "get_tome_examples") {
        const topic = (args as any).topic;
        if (topic === "file_write") {
            const example = getFileWriteExample();
            return {
                content: [
                    { type: "text", text: "## metadata.yml\n" + example.metadata },
                    { type: "text", text: "## main.eldritch\n" + example.script }
                ]
            };
        } else if (topic === "persist_service") {
             const example = getPersistServiceExample();
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
