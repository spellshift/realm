import { ExtensionContext } from 'vscode';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
} from 'vscode-languageclient';

let client: LanguageClient;

export function activate(context: ExtensionContext) {
    // Otherwise to spawn the server
    let serverOptions: ServerOptions = { command: "eldritch-lang", args: ["--lsp"] };

    // Options to control the language client
    let clientOptions: LanguageClientOptions = {
        // Register the server for Starlark documents
        documentSelector: [{ scheme: 'file', language: 'eldritch' }],
    };

    // Create the language client and start the client.
    client = new LanguageClient(
        'Eldritch',
        'Eldritch language server',
        serverOptions,
        clientOptions
    );

    // Start the client. This will also launch the server
    client.start();
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}