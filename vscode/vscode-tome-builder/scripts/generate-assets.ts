import * as fs from 'fs';
import * as path from 'path';

// Script is located in vscode/vscode-tome-builder/scripts/
// We need to go up 3 levels to reach the repo root: scripts -> vscode-tome-builder -> vscode -> root
const REPO_ROOT = path.resolve(__dirname, '../../..');
const EXTENSION_ROOT = path.resolve(__dirname, '..');

const TOMES_DOC_PATH = path.join(REPO_ROOT, 'docs/_docs/user-guide/tomes.md');
const ELDRITCH_DOC_PATH = path.join(REPO_ROOT, 'docs/_docs/user-guide/eldritch.md');

const MCP_SERVER_SRC = path.join(EXTENSION_ROOT, 'mcp-server/src');
const DOCS_TS_PATH = path.join(MCP_SERVER_SRC, 'docs.ts');
const EXAMPLES_TS_PATH = path.join(MCP_SERVER_SRC, 'examples.ts');

const FILE_WRITE_DIR = path.join(REPO_ROOT, 'tavern/tomes/file_write');
const PERSIST_SERVICE_DIR = path.join(REPO_ROOT, 'tavern/tomes/persist_service');

function readFile(filePath: string): string {
    try {
        return fs.readFileSync(filePath, 'utf-8');
    } catch (error) {
        console.error(`Error reading file ${filePath}:`, error);
        process.exit(1);
    }
}

function escapeContent(content: string): string {
    // Escape backticks and dollar signs to be safe inside a template literal
    return content.replace(/\\/g, '\\\\').replace(/`/g, '\\`').replace(/\$/g, '\\$');
}

function generateDocs() {
    console.log('Generating docs.ts...');
    const tomesDoc = readFile(TOMES_DOC_PATH);
    const eldritchDoc = readFile(ELDRITCH_DOC_PATH);

    const safeTomesDoc = escapeContent(tomesDoc);
    const safeEldritchDoc = escapeContent(eldritchDoc);

    const content = `export const TOMES_DOC = \`${safeTomesDoc}\`;

export const ELDRITCH_DOC = \`${safeEldritchDoc}\`;
`;

    fs.writeFileSync(DOCS_TS_PATH, content);
    console.log(`Wrote to ${DOCS_TS_PATH}`);
}

function generateExamples() {
    console.log('Generating examples.ts...');

    const examples = [
        {
            name: 'FILE_WRITE_EXAMPLE',
            dir: FILE_WRITE_DIR
        },
        {
            name: 'PERSIST_SERVICE_EXAMPLE',
            dir: PERSIST_SERVICE_DIR
        }
    ];

    let content = '';

    for (const example of examples) {
        const metadata = readFile(path.join(example.dir, 'metadata.yml'));
        const script = readFile(path.join(example.dir, 'main.eldritch'));

        const safeMetadata = escapeContent(metadata);
        const safeScript = escapeContent(script);

        content += `export const ${example.name} = {
    metadata: \`${safeMetadata}\`,
    script: \`${safeScript}\`
};

`;
    }

    fs.writeFileSync(EXAMPLES_TS_PATH, content);
    console.log(`Wrote to ${EXAMPLES_TS_PATH}`);
}

function main() {
    if (!fs.existsSync(MCP_SERVER_SRC)) {
        console.error(`Directory not found: ${MCP_SERVER_SRC}`);
        // Create directory if it doesn't exist?
        // The instructions imply generating files in existing src.
        // But if it's missing, maybe we should create it.
        // Given earlier exploration, it exists.
        process.exit(1);
    }
    generateDocs();
    generateExamples();
}

main();
