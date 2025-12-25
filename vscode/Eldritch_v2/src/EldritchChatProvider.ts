import * as vscode from 'vscode';
import * as path from 'path';
import { ContextManager } from './ContextManager';
import { GeminiClient } from './GeminiClient';

export class EldritchChatProvider implements vscode.WebviewViewProvider {
    public static readonly viewType = 'eldritch.chatView';
    private _view?: vscode.WebviewView;

    constructor(
        private readonly _extensionUri: vscode.Uri,
        private readonly _contextManager: ContextManager,
        private readonly _geminiClient: GeminiClient
    ) { }

    public resolveWebviewView(
        webviewView: vscode.WebviewView,
        context: vscode.WebviewViewResolveContext,
        _token: vscode.CancellationToken,
    ) {
        this._view = webviewView;

        webviewView.webview.options = {
            enableScripts: true,
            localResourceRoots: [this._extensionUri]
        };

        webviewView.webview.html = this._getHtmlForWebview(webviewView.webview);

        webviewView.webview.onDidReceiveMessage(async (data) => {
            switch (data.type) {
                case 'sendMessage': {
                    await this.handleUserMessage(data.value);
                    break;
                }
                case 'saveTome': {
                    await this.saveTome(data.metadata, data.code);
                    break;
                }
            }
        });
    }

    private async handleUserMessage(message: string) {
        if (!this._view) { return; }

        // Show user message
        this._view.webview.postMessage({ type: 'addMessage', role: 'user', content: message });
        this._view.webview.postMessage({ type: 'setLoading', value: true });

        try {
            const context = await this._contextManager.getContext();
            const response = await this._geminiClient.generateTome(message, context);

            // Parse response
            const metadataMatch = response.match(/---BEGIN METADATA---([\s\S]*?)---END METADATA---/);
            const codeMatch = response.match(/---BEGIN CODE---([\s\S]*?)---END CODE---/);

            if (metadataMatch && codeMatch) {
                const metadata = metadataMatch[1].trim();
                const code = codeMatch[1].trim();

                this._view.webview.postMessage({
                    type: 'addTomePreview',
                    metadata: metadata,
                    code: code
                });
            } else {
                // Fallback to raw text if format is wrong
                this._view.webview.postMessage({ type: 'addMessage', role: 'ai', content: response });
            }

        } catch (error: any) {
            this._view.webview.postMessage({ type: 'addMessage', role: 'error', content: error.message });
        } finally {
            this._view.webview.postMessage({ type: 'setLoading', value: false });
        }
    }

    private async saveTome(metadata: string, code: string) {
        // Extract name from metadata
        const nameMatch = metadata.match(/^name:\s*(.+)$/m);
        let tomeName = 'NewTome';
        if (nameMatch) {
            tomeName = nameMatch[1].trim().replace(/['"]/g, ''); // Basic cleanup
        }

        // Sanitize filename
        const safeName = tomeName.replace(/[^a-zA-Z0-9-_]/g, '_');

        const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
        if (!workspaceFolder) {
            vscode.window.showErrorMessage('No workspace open.');
            return;
        }

        const generatedTomesDir = vscode.Uri.joinPath(workspaceFolder.uri, 'Generated Tomes');
        const tomeDir = vscode.Uri.joinPath(generatedTomesDir, safeName);

        try {
            await vscode.workspace.fs.createDirectory(tomeDir);

            const metadataUri = vscode.Uri.joinPath(tomeDir, 'metadata.yml');
            const codeUri = vscode.Uri.joinPath(tomeDir, 'main.eldritch');

            await vscode.workspace.fs.writeFile(metadataUri, Buffer.from(metadata, 'utf8'));
            await vscode.workspace.fs.writeFile(codeUri, Buffer.from(code, 'utf8'));

            vscode.window.showInformationMessage(`Tome '${tomeName}' saved to Generated Tomes/${safeName}`);

            // Open the main file
            const doc = await vscode.workspace.openTextDocument(codeUri);
            await vscode.window.showTextDocument(doc);

        } catch (e: any) {
            vscode.window.showErrorMessage(`Failed to save Tome: ${e.message}`);
        }
    }

    private _getHtmlForWebview(webview: vscode.Webview) {
        return `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Eldritch AI</title>
    <style>
        body { font-family: var(--vscode-font-family); padding: 10px; color: var(--vscode-foreground); }
        .message { margin-bottom: 10px; padding: 8px; border-radius: 4px; white-space: pre-wrap; }
        .user { background-color: var(--vscode-list-hoverBackground); align-self: flex-end; }
        .ai { background-color: var(--vscode-editor-background); border: 1px solid var(--vscode-widget-border); }
        .error { color: var(--vscode-errorForeground); border: 1px solid var(--vscode-errorForeground); }
        .tome-preview { border: 1px solid var(--vscode-focusBorder); padding: 10px; margin-top: 10px; }
        pre { background: var(--vscode-textBlockQuote-background); padding: 5px; overflow-x: auto; }
        textarea { width: 100%; height: 60px; background: var(--vscode-input-background); color: var(--vscode-input-foreground); border: 1px solid var(--vscode-input-border); box-sizing: border-box; }
        button { background: var(--vscode-button-background); color: var(--vscode-button-foreground); border: none; padding: 6px 12px; cursor: pointer; margin-top: 5px; }
        button:hover { background: var(--vscode-button-hoverBackground); }
        #chat-container { display: flex; flex-direction: column; height: calc(100vh - 120px); overflow-y: auto; padding-bottom: 10px; }
        #input-container { position: fixed; bottom: 0; left: 0; right: 0; padding: 10px; background: var(--vscode-sideBar-background); border-top: 1px solid var(--vscode-widget-border); }
    </style>
</head>
<body>
    <div id="chat-container"></div>
    <div id="input-container">
        <textarea id="prompt-input" placeholder="Describe the Tome you want to create..."></textarea>
        <button id="send-btn">Generate Tome</button>
    </div>

    <script>
        const vscode = acquireVsCodeApi();
        const chatContainer = document.getElementById('chat-container');
        const promptInput = document.getElementById('prompt-input');
        const sendBtn = document.getElementById('send-btn');

        sendBtn.addEventListener('click', () => {
            const text = promptInput.value;
            if (text) {
                vscode.postMessage({ type: 'sendMessage', value: text });
                promptInput.value = '';
            }
        });

        // Allow Ctrl+Enter to send
        promptInput.addEventListener('keydown', (e) => {
            if (e.ctrlKey && e.key === 'Enter') {
                sendBtn.click();
            }
        });

        window.addEventListener('message', event => {
            const message = event.data;
            switch (message.type) {
                case 'addMessage':
                    const div = document.createElement('div');
                    div.className = 'message ' + message.role;
                    div.textContent = message.content;
                    chatContainer.appendChild(div);
                    break;
                case 'addTomePreview':
                    const container = document.createElement('div');
                    container.className = 'tome-preview';
                    container.innerHTML = '<h3>Generated Tome</h3>' +
                        '<h4>metadata.yml</h4><pre>' + escapeHtml(message.metadata) + '</pre>' +
                        '<h4>main.eldritch</h4><pre>' + escapeHtml(message.code) + '</pre>';

                    const saveBtn = document.createElement('button');
                    saveBtn.textContent = 'Save to Generated Tomes';
                    saveBtn.onclick = () => {
                        vscode.postMessage({ type: 'saveTome', metadata: message.metadata, code: message.code });
                    };

                    container.appendChild(saveBtn);
                    chatContainer.appendChild(container);
                    break;
                case 'setLoading':
                    if (message.value) {
                         const loading = document.createElement('div');
                         loading.id = 'loading-indicator';
                         loading.className = 'message ai';
                         loading.textContent = 'Generating... (this may take a moment)';
                         chatContainer.appendChild(loading);
                    } else {
                        const loading = document.getElementById('loading-indicator');
                        if (loading) loading.remove();
                    }
                    break;
            }
            chatContainer.scrollTop = chatContainer.scrollHeight;
        });

        function escapeHtml(unsafe) {
            return unsafe
                 .replace(/&/g, "&amp;")
                 .replace(/</g, "&lt;")
                 .replace(/>/g, "&gt;")
                 .replace(/"/g, "&quot;")
                 .replace(/'/g, "&#039;");
        }
    </script>
</body>
</html>`;
    }
}
