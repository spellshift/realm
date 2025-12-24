import { ExtensionContext, window } from 'vscode';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
} from 'vscode-languageclient';
import * as path from 'path';
import * as fs from 'fs';

let client: LanguageClient;

export function activate(context: ExtensionContext) {
    let serverPath = getServerPath(context);

    if (!serverPath) {
        window.showErrorMessage("Eldritch LSP binary not found. Please install it or build it using `cargo build -p eldritch-lsp --release`.");
        return;
    }

    let serverOptions: ServerOptions = {
        command: serverPath,
        args: []
    };

    // Options to control the language client
    let clientOptions: LanguageClientOptions = {
        // Register the server for Eldritch documents
        documentSelector: [{ scheme: 'file', language: 'eldritch' }],
        outputChannelName: 'Eldritch LSP',
        // Reveal output channel if there is an error
        revealOutputChannelOn: 4 // RevealOutputChannelOn.Error
    };

    // Create the language client and start the client.
    client = new LanguageClient(
        'Eldritch',
        'Eldritch Language Server',
        serverOptions,
        clientOptions
    );

    client.start();

    client.onReady().then(() => {
        // Server started successfully
        console.log("Eldritch LSP started.");
    }, (error) => {
        window.showErrorMessage(`Eldritch LSP failed to initialize: ${error}`);
        console.error("Eldritch LSP initialization error:", error);
    });
}

function getServerPath(context: ExtensionContext): string | undefined {
    // 1. Check bundled binary in bin/
    let bundledPath = context.asAbsolutePath(path.join('bin', 'eldritch-lsp'));
    if (process.platform === 'win32') {
        bundledPath += '.exe';
    }

    if (fs.existsSync(bundledPath)) {
        return bundledPath;
    }

    // 2. Check PATH (naive check by returning command name)
    // Actually, if we return "eldritch-lsp", vscode-languageclient will look in PATH.
    // But we want to be descriptive if it fails.
    // Let's assume if bundled is missing, we try PATH.
    return "eldritch-lsp";
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
