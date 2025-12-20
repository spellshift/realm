import * as vscode from 'vscode';
import * as path from 'path';

export class ContextManager {
    public async getContext(): Promise<string> {
        const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
        if (!workspaceFolder) {
            return "No workspace folder open. Please open the Realm repository root.";
        }

        const docsPath = path.join(workspaceFolder.uri.fsPath, 'docs', '_docs', 'user-guide');
        const tomesPath = path.join(workspaceFolder.uri.fsPath, 'tavern', 'tomes');

        let context = "You are an expert in writing Eldritch Tomes for the Realm red-teaming platform.\n\n";

        // Read Documentation
        try {
            const eldritchDoc = await this.readFile(path.join(docsPath, 'eldritch.md'));
            const tomesDoc = await this.readFile(path.join(docsPath, 'tomes.md'));
            context += "## Language Documentation\n" + eldritchDoc + "\n\n";
            context += "## Tomes Documentation\n" + tomesDoc + "\n\n";
        } catch (e) {
            console.error("Error reading docs:", e);
            context += "Warning: Could not read documentation files from the workspace. Ensure you have the full repo open.\n\n";
        }

        // Read Examples
        context += "## Example Tomes\n";
        const examples = ['process_list', 'http_get_file'];

        for (const example of examples) {
            try {
                const examplePath = path.join(tomesPath, example);
                const metadata = await this.readFile(path.join(examplePath, 'metadata.yml'));
                const main = await this.readFile(path.join(examplePath, 'main.eldritch'));

                context += `### Example: ${example}\n`;
                context += `#### metadata.yml\n\`\`\`yaml\n${metadata}\n\`\`\`\n`;
                context += `#### main.eldritch\n\`\`\`python\n${main}\n\`\`\`\n\n`;
            } catch (e) {
                console.error(`Error reading example ${example}:`, e);
            }
        }

        return context;
    }

    private async readFile(filePath: string): Promise<string> {
        const uri = vscode.Uri.file(filePath);
        try {
            const bytes = await vscode.workspace.fs.readFile(uri);
            return Buffer.from(bytes).toString('utf8');
        } catch (error) {
            console.error(`Failed to read file: ${filePath}`, error);
            throw error;
        }
    }
}
