import { ExtensionContext, window } from 'vscode';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
} from 'vscode-languageclient';

let client: LanguageClient;

export function activate(context: ExtensionContext) {
    // We assume eldritch-lsp is in the PATH.
    let serverOptions: ServerOptions = {
        command: "eldritch-lsp",
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

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
