import * as fs from "fs";
import * as path from "path";
import { spawnSync } from "child_process";
import * as vscode from "vscode";
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
} from "vscode-languageclient";

let client: LanguageClient | undefined;
let outputChannel: vscode.OutputChannel | undefined;

function canExecuteBinary(binPath: string): boolean {
    try {
        const res = spawnSync(binPath, ["--version"], {
            timeout: 3000,
            encoding: "utf8",
        });
        return !res.error && res.status === 0;
    } catch {
        return false;
    }
}

function resolveServerOptions(context: vscode.ExtensionContext, channel: vscode.OutputChannel): ServerOptions {
    const config = vscode.workspace.getConfiguration("eldritch");
    const configuredPath = (config.get<string>("server.path") || "").trim();

    const candidates: string[] = [];
    if (configuredPath) {
        candidates.push(configuredPath);
    }

    const extSuffix = process.platform === "win32" ? ".exe" : "";
    const platformArchBin = path.join(
        context.extensionPath,
        "bin",
        `eldritch-lsp-${process.platform}-${process.arch}${extSuffix}`
    );
    const defaultBundledBin = path.join(
        context.extensionPath,
        "bin",
        `eldritch-lsp${extSuffix}`
    );
    candidates.push(platformArchBin, defaultBundledBin);

    const workspaceFolders = vscode.workspace.workspaceFolders || [];
    for (const folder of workspaceFolders) {
        const root = folder.uri.fsPath;
        candidates.push(
            path.join(root, "implants", "target", "release", `eldritch-lsp${extSuffix}`),
            path.join(root, "implants", "target", "debug", `eldritch-lsp${extSuffix}`)
        );
    }

    candidates.push(`eldritch-lsp${extSuffix}`);

    for (const candidate of candidates) {
        const isPathLookup = !candidate.includes(path.sep) && !candidate.includes("/");
        if (isPathLookup || fs.existsSync(candidate)) {
            if (!isPathLookup) {
                try {
                    fs.chmodSync(candidate, 0o755);
                } catch {
                    // ignore chmod errors on read-only filesystems
                }
            }
            if (canExecuteBinary(candidate)) {
                channel.appendLine(`[eldritch] Using native Rust language server: ${candidate}`);
                return {
                    command: candidate,
                    args: ["--lsp"],
                };
            }
        }
    }

    const fallbackServer = path.join(context.extensionPath, "client", "out", "server.js");
    channel.appendLine(
        `[eldritch] Native eldritch-lsp binary not available for ${process.platform}-${process.arch}; using bundled Node language server: ${fallbackServer}`
    );
    return {
        command: process.execPath,
        args: [fallbackServer, "--stdio"],
    };
}

async function startLanguageClient(context: vscode.ExtensionContext): Promise<void> {
    if (!outputChannel) {
        outputChannel = vscode.window.createOutputChannel("Eldritch Language Server");
        context.subscriptions.push(outputChannel);
    }

    const serverOptions = resolveServerOptions(context, outputChannel);

    const clientOptions: LanguageClientOptions = {
        documentSelector: [
            { scheme: "file", language: "eldritch" },
            { scheme: "untitled", language: "eldritch" },
        ],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher("**/{metadata.yml,metadata.yaml,*.eldritch,*.tome}"),
        },
        outputChannel,
    };

    client = new LanguageClient(
        "eldritch",
        "Eldritch Language Server",
        serverOptions,
        clientOptions
    );

    context.subscriptions.push(client.start());
    await client.onReady();
}

export async function activate(context: vscode.ExtensionContext): Promise<void> {
    await startLanguageClient(context);

    context.subscriptions.push(
        vscode.commands.registerCommand("eldritch.restartServer", async () => {
            if (client) {
                await client.stop();
                client = undefined;
            }
            await startLanguageClient(context);
            vscode.window.showInformationMessage("Eldritch Language Server restarted.");
        }),
        vscode.commands.registerCommand("eldritch.showOutputChannel", () => {
            outputChannel?.show(true);
        }),
        vscode.commands.registerCommand("eldritch.checkFile", async () => {
            const editor = vscode.window.activeTextEditor;
            if (!editor || editor.document.languageId !== "eldritch") {
                vscode.window.showWarningMessage("Open an Eldritch (.eldritch or .tome) file to run linting.");
                return;
            }
            const diagnostics = vscode.languages.getDiagnostics(editor.document.uri);
            if (diagnostics.length === 0) {
                vscode.window.showInformationMessage("Eldritch: No lint issues found in current file.");
            } else {
                vscode.commands.executeCommand("workbench.actions.view.problems");
            }
        })
    );
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
