"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const index_js_1 = require("@modelcontextprotocol/sdk/server/index.js");
const stdio_js_1 = require("@modelcontextprotocol/sdk/server/stdio.js");
const types_js_1 = require("@modelcontextprotocol/sdk/types.js");
const docs_1 = require("./docs");
const server = new index_js_1.Server({
    name: "tome-builder",
    version: "1.0.0",
}, {
    capabilities: {
        tools: {},
    },
});
// Define Tools
const getDocumentationTool = {
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
const validateTomeStructureTool = {
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
server.setRequestHandler(types_js_1.ListToolsRequestSchema, async () => {
    return {
        tools: [getDocumentationTool, validateTomeStructureTool]
    };
});
server.setRequestHandler(types_js_1.CallToolRequestSchema, async (request) => {
    const { name, arguments: args } = request.params;
    if (name === "get_documentation") {
        const topic = args.topic;
        if (topic === "tomes") {
            return {
                content: [{ type: "text", text: docs_1.TOMES_DOC }],
            };
        }
        else if (topic === "eldritch") {
            return {
                content: [{ type: "text", text: docs_1.ELDRITCH_DOC }],
            };
        }
        return { content: [], isError: true };
    }
    if (name === "validate_tome_structure") {
        const yaml_content = args.yaml_content;
        const script_content = args.script_content;
        const errors = [];
        if (!yaml_content.includes("name:"))
            errors.push("metadata.yml missing 'name'");
        if (!yaml_content.includes("description:"))
            errors.push("metadata.yml missing 'description'");
        if (!yaml_content.includes("tactic:"))
            errors.push("metadata.yml missing 'tactic'");
        if (!script_content.includes("def main"))
            errors.push("main.eldritch missing 'def main' function");
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
    const transport = new stdio_js_1.StdioServerTransport();
    await server.connect(transport);
    console.error("Tome MCP Server running on stdio");
}
main().catch((error) => {
    console.error("Fatal error in main():", error);
    process.exit(1);
});
//# sourceMappingURL=index.js.map