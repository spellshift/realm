import * as vscode from 'vscode';
import { McpClientService } from '../services/McpClientService';
import { LlmService } from '../services/LlmService';

export class ChatPanel implements vscode.WebviewViewProvider {
    public static readonly viewType = 'tome-builder-chat';
    private _view?: vscode.WebviewView;
    private _llmService?: LlmService;

    constructor(
        private readonly _extensionUri: vscode.Uri,
        private readonly _context: vscode.ExtensionContext,
        private readonly _mcpService: McpClientService
    ) {}

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
                case 'init': {
                    await this._handleInit();
                    break;
                }
                case 'userMessage': {
                    await this._handleUserMessage(data.value);
                    break;
                }
                case 'saveFile': {
                    await this._handleSaveFile(data.content, data.language);
                    break;
                }
                case 'modelSelected': {
                    await this._handleModelSelected(data.value);
                    break;
                }
                case 'log': {
                    console.log(`Webview Log: ${data.value}`);
                    break;
                }
            }
        });
    }

    private async _handleInit() {
         if (!this._view) return;
         try {
             // Try to init service to get models
             const config = vscode.workspace.getConfiguration('tomeBuilder');
             const apiKey = config.get<string>('llm.apiKey');
             const savedModel = config.get<string>('llm.model') || 'gemini-2.0-flash-exp';

             if (!apiKey) {
                 this._view.webview.postMessage({ type: 'addMessage', role: 'system', content: 'Please set your Gemini API Key in Settings (Tome Builder > Llm > Api Key).' });
                 return;
             }

             const client = this._mcpService.getClient();
             if (!client) {
                 this._view.webview.postMessage({ type: 'addMessage', role: 'error', content: 'MCP Client not connected. Check extension logs.' });
                 return;
             }

             if (!this._llmService) {
                this._llmService = new LlmService(apiKey, client, savedModel);
             }

             // Fetch models
             const models = await this._llmService.listAvailableModels();
             this._view.webview.postMessage({ type: 'setModels', models: models, current: savedModel });

         } catch (e: any) {
             console.error("Init failed:", e);
         }
    }

    private async _handleModelSelected(modelName: string) {
        if (this._llmService) {
            this._llmService.setModel(modelName);
            // Optionally save to config
            await vscode.workspace.getConfiguration('tomeBuilder').update('llm.model', modelName, vscode.ConfigurationTarget.Global);
            if (this._view) {
                this._view.webview.postMessage({ type: 'addMessage', role: 'system', content: `Switched to model: ${modelName}` });
            }
        }
    }

    private async _handleSaveFile(content: string, language: string) {
        let defaultName = 'untitled';
        if (language === 'yaml' || language === 'yml') defaultName = 'metadata.yml';
        if (language === 'python' || language === 'eldritch') defaultName = 'main.eldritch';

        const uri = await vscode.window.showSaveDialog({
            defaultUri: vscode.Uri.file(defaultName),
            saveLabel: 'Save Tome File',
            filters: {
                'Tome Files': ['eldritch', 'yml', 'yaml'],
                'All Files': ['*']
            }
        });

        if (uri) {
            await vscode.workspace.fs.writeFile(uri, Buffer.from(content, 'utf8'));
            vscode.window.showInformationMessage(`Saved to ${uri.fsPath}`);
        }
    }

    private async _handleUserMessage(message: string) {
        if (!this._view) return;

        try {
            const config = vscode.workspace.getConfiguration('tomeBuilder');
            const apiKey = config.get<string>('llm.apiKey');
            // We use the service's current model, no need to re-read config unless init failed.

            if (!apiKey) {
                this._view.webview.postMessage({ type: 'addMessage', role: 'system', content: 'Please set your Gemini API Key in Settings (Tome Builder > Llm > Api Key).' });
                return;
            }

            if (!this._llmService) {
                const client = this._mcpService.getClient();
                if (!client) {
                     this._view.webview.postMessage({ type: 'addMessage', role: 'error', content: 'MCP Client not connected. Check extension logs.' });
                     return;
                }
                const model = config.get<string>('llm.model') || 'gemini-2.0-flash-exp';
                this._llmService = new LlmService(apiKey, client, model);
            }

            this._view.webview.postMessage({ type: 'setLoading', value: true });

            const response = await this._llmService.sendMessage(message);

            this._view.webview.postMessage({ type: 'addMessage', role: 'assistant', content: response });

        } catch (e: any) {
            console.error(e);
            let errorMsg = `Error: ${e.message || e}`;
            if (e.message && e.message.includes("404")) {
                errorMsg += "\n\nThe selected model might not be available. Please try selecting a different model from the dropdown above.";
            }
            this._view.webview.postMessage({ type: 'addMessage', role: 'error', content: errorMsg });
        } finally {
            this._view.webview.postMessage({ type: 'setLoading', value: false });
        }
    }

    private _getHtmlForWebview(webview: vscode.Webview) {
        return `<!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>Tome Builder</title>
            <style>
                body {
                    font-family: var(--vscode-font-family);
                    padding: 10px;
                    color: var(--vscode-foreground);
                    background-color: var(--vscode-editor-background);
                    display: flex;
                    flex-direction: column;
                    height: 100vh;
                    box-sizing: border-box;
                    margin: 0;
                }
                #header {
                    display: flex;
                    justify-content: space-between;
                    align-items: center;
                    margin-bottom: 10px;
                    padding-bottom: 5px;
                    border-bottom: 1px solid var(--vscode-widget-border);
                }
                #model-select {
                    background-color: var(--vscode-dropdown-background);
                    color: var(--vscode-dropdown-foreground);
                    border: 1px solid var(--vscode-dropdown-border);
                    padding: 2px 4px;
                }
                #chat-container {
                    flex-grow: 1;
                    overflow-y: auto;
                    display: flex;
                    flex-direction: column;
                    gap: 10px;
                    margin-bottom: 10px;
                    padding-right: 5px;
                }
                .message {
                    padding: 8px 12px;
                    border-radius: 6px;
                    max-width: 90%;
                    word-wrap: break-word;
                    white-space: pre-wrap;
                }
                .user {
                    align-self: flex-end;
                    background-color: var(--vscode-button-background);
                    color: var(--vscode-button-foreground);
                }
                .assistant {
                    align-self: flex-start;
                    background-color: var(--vscode-editor-inactiveSelectionBackground);
                }
                .error {
                    color: var(--vscode-errorForeground);
                    border: 1px solid var(--vscode-errorForeground);
                }
                .system {
                    font-style: italic;
                    color: var(--vscode-descriptionForeground);
                }
                .code-block {
                    background-color: var(--vscode-textCodeBlock-background);
                    border: 1px solid var(--vscode-widget-border);
                    border-radius: 4px;
                    margin-top: 5px;
                    overflow: hidden;
                }
                .code-header {
                    display: flex;
                    justify-content: space-between;
                    background-color: var(--vscode-editor-background);
                    padding: 4px 8px;
                    border-bottom: 1px solid var(--vscode-widget-border);
                    font-size: 0.8em;
                }
                pre {
                    margin: 0;
                    padding: 8px;
                    overflow-x: auto;
                }
                code {
                    font-family: var(--vscode-editor-font-family);
                }
                #input-container {
                    display: flex;
                    gap: 5px;
                    padding-top: 10px;
                    border-top: 1px solid var(--vscode-widget-border);
                }
                textarea {
                    flex-grow: 1;
                    background-color: var(--vscode-input-background);
                    color: var(--vscode-input-foreground);
                    border: 1px solid var(--vscode-input-border);
                    resize: vertical;
                    min-height: 40px;
                    font-family: var(--vscode-font-family);
                }
                button {
                    background-color: var(--vscode-button-background);
                    color: var(--vscode-button-foreground);
                    border: none;
                    padding: 4px 12px;
                    cursor: pointer;
                    border-radius: 2px;
                }
                button:disabled {
                    opacity: 0.5;
                }
            </style>
        </head>
        <body>
            <div id="header">
                <h3>Tome Builder</h3>
                <select id="model-select">
                    <option value="" disabled selected>Loading models...</option>
                </select>
            </div>
            <div id="chat-container"></div>
            <div id="input-container">
                <textarea id="message-input" placeholder="Describe the Tome you want to create..."></textarea>
                <button id="send-btn">Send</button>
            </div>
            <script>
                const vscode = acquireVsCodeApi();
                const chatContainer = document.getElementById('chat-container');
                const messageInput = document.getElementById('message-input');
                const sendBtn = document.getElementById('send-btn');
                const modelSelect = document.getElementById('model-select');

                function formatContent(content) {
                    if (!content) return '';
                    // Replace code blocks with custom HTML
                    return content.replace(/\\\`\\\`\\\`(\\w+)?\\n([\\s\\S]*?)\\\`\\\`\\\`/g, (match, lang, code) => {
                        const escapedCode = code.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
                        return \`<div class="code-block" data-lang="\${lang || ''}">
                            <div class="code-header">
                                <span>\${lang || 'Code'}</span>
                                <button onclick="saveCode(this)">Save</button>
                            </div>
                            <pre><code>\${escapedCode}</code></pre>
                            <textarea style="display:none">\${code}</textarea>
                        </div>\`;
                    });
                }

                // Global function for save button
                window.saveCode = function(btn) {
                    const block = btn.closest('.code-block');
                    const lang = block.dataset.lang;
                    const code = block.querySelector('textarea').value;
                    vscode.postMessage({ type: 'saveFile', content: code, language: lang });
                };

                function addMessage(role, content) {
                    const div = document.createElement('div');
                    div.className = 'message ' + role;

                    if (role === 'assistant') {
                        div.innerHTML = formatContent(content);
                    } else {
                        div.textContent = content;
                    }

                    chatContainer.appendChild(div);
                    chatContainer.scrollTop = chatContainer.scrollHeight;
                }

                sendBtn.addEventListener('click', () => {
                    const text = messageInput.value.trim();
                    if (!text) return;

                    addMessage('user', text);
                    vscode.postMessage({ type: 'userMessage', value: text });
                    messageInput.value = '';
                });

                modelSelect.addEventListener('change', (e) => {
                    const model = e.target.value;
                    vscode.postMessage({ type: 'modelSelected', value: model });
                });

                window.addEventListener('message', event => {
                    const message = event.data;
                    switch (message.type) {
                        case 'addMessage':
                            addMessage(message.role, message.content);
                            break;
                        case 'setLoading':
                            if (message.value) {
                                sendBtn.disabled = true;
                                sendBtn.textContent = '...';
                            } else {
                                sendBtn.disabled = false;
                                sendBtn.textContent = 'Send';
                            }
                            break;
                        case 'setModels':
                            modelSelect.innerHTML = '';
                            message.models.forEach(model => {
                                const option = document.createElement('option');
                                option.value = model;
                                option.textContent = model;
                                if (model === message.current) {
                                    option.selected = true;
                                }
                                modelSelect.appendChild(option);
                            });
                            break;
                    }
                });

                // Init
                vscode.postMessage({ type: 'init' });
            </script>
        </body>
        </html>`;
    }
}
