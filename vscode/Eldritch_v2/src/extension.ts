import * as vscode from 'vscode';
import { EldritchChatProvider } from './EldritchChatProvider';
import { ContextManager } from './ContextManager';
import { GeminiClient } from './GeminiClient';

export function activate(context: vscode.ExtensionContext) {
    const contextManager = new ContextManager();
    const geminiClient = new GeminiClient();

    const provider = new EldritchChatProvider(context.extensionUri, contextManager, geminiClient);

    context.subscriptions.push(
        vscode.window.registerWebviewViewProvider(EldritchChatProvider.viewType, provider)
    );
}

export function deactivate() {}
