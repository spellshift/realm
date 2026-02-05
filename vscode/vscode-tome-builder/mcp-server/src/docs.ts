import * as fs from 'fs';
import * as path from 'path';

function findRepoRoot(): string | null {
    let currentDir = __dirname;
    while (currentDir !== path.parse(currentDir).root) {
        if (fs.existsSync(path.join(currentDir, 'go.mod'))) {
            // Found the repo root (assuming go.mod exists in root, which it does in this repo)
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

export function getTomesDoc(): string {
    return readDocFile('docs/_docs/user-guide/tomes.md');
}

export function getEldritchDoc(): string {
    return readDocFile('docs/_docs/user-guide/eldritch.md');
}
