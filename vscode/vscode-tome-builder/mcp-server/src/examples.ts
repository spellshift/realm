import * as fs from 'fs';
import * as path from 'path';

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

export function getFileWriteExample() {
    return readExample('file_write');
}

export function getPersistServiceExample() {
    return readExample('persist_service');
}
