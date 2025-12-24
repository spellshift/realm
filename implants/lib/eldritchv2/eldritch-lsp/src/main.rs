use anyhow::Result;
use crossbeam_channel::Sender;
use eldritch_core::{Parser, Lexer};
// use eldritchv2::Interpreter as V2Interpreter;
use lsp_server::{Connection, Message, Notification, Request, Response};
use lsp_types::{
    notification::{DidChangeTextDocument, DidOpenTextDocument, DidCloseTextDocument, PublishDiagnostics, Notification as LspNotification},
    request::{Completion, Initialize, Request as LspRequest},
    CompletionItem, CompletionItemKind, CompletionParams, CompletionResponse,
    Diagnostic, DiagnosticSeverity, DidChangeTextDocumentParams, DidOpenTextDocumentParams,
    InitializeParams, Position, Range, ServerCapabilities, TextDocumentSyncCapability,
    TextDocumentSyncKind, Url, DidCloseTextDocumentParams, PublishDiagnosticsParams,
    InitializeResult,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

mod linter;
use linter::Linter;

struct ServerState {
    // Map of document URI to (version, content)
    documents: HashMap<Url, (i32, String)>,
    linter: Linter,
}

impl ServerState {
    fn new() -> Self {
        Self {
            documents: HashMap::new(),
            linter: Linter::new(),
        }
    }
}

fn main() -> Result<()> {
    // Initialize logging
    let _ = simplelog::WriteLogger::init(
        simplelog::LevelFilter::Info,
        simplelog::Config::default(),
        std::fs::File::create("/tmp/eldritch-lsp.log").unwrap_or_else(|_| std::fs::File::create("eldritch-lsp.log").unwrap()),
    );

    log::info!("Starting Eldritch LSP...");

    let (connection, io_threads) = Connection::stdio();

    // Run initialization handshake
    let (id, params) = connection.initialize_start()?;
    let _init_params: InitializeParams = serde_json::from_value(params)?;

    let capabilities = ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        completion_provider: Some(lsp_types::CompletionOptions {
            resolve_provider: Some(false),
            trigger_characters: Some(vec![".".to_string()]),
            ..Default::default()
        }),
        ..ServerCapabilities::default()
    };

    // Manually constructing JSON result to ensure correct structure
    let result = serde_json::json!({
        "capabilities": capabilities,
        "serverInfo": {
            "name": "eldritch-lsp",
            "version": "0.1.0"
        }
    });

    connection.initialize_finish(id, result)?;

    let state = Arc::new(Mutex::new(ServerState::new()));

    log::info!("Eldritch LSP initialized.");

    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    return Ok(());
                }

                let state = state.clone();
                let sender = connection.sender.clone();

                match req.method.as_str() {
                    Completion::METHOD => {
                        let params: CompletionParams = serde_json::from_value(req.params).unwrap();
                        let resp = handle_completion(state, params);
                        let result = serde_json::to_value(&resp).unwrap();
                        sender.send(Message::Response(Response {
                            id: req.id,
                            result: Some(result),
                            error: None,
                        })).unwrap();
                    }
                    _ => {
                         // forward unknown requests or ignore
                    }
                }
            }
            Message::Response(_) => {}
            Message::Notification(not) => {
                let state = state.clone();
                let sender = connection.sender.clone();

                match not.method.as_str() {
                    DidOpenTextDocument::METHOD => {
                        let params: DidOpenTextDocumentParams = serde_json::from_value(not.params).unwrap();
                        handle_did_open(state, sender, params);
                    }
                    DidChangeTextDocument::METHOD => {
                        let params: DidChangeTextDocumentParams = serde_json::from_value(not.params).unwrap();
                        handle_did_change(state, sender, params);
                    }
                    DidCloseTextDocument::METHOD => {
                         let params: DidCloseTextDocumentParams = serde_json::from_value(not.params).unwrap();
                         let mut state = state.lock().unwrap();
                         state.documents.remove(&params.text_document.uri);
                    }
                    _ => {}
                }
            }
        }
    }

    io_threads.join()?;
    Ok(())
}

fn handle_did_open(state: Arc<Mutex<ServerState>>, sender: Sender<Message>, params: DidOpenTextDocumentParams) {
    let mut s = state.lock().unwrap();
    s.documents.insert(params.text_document.uri.clone(), (params.text_document.version, params.text_document.text.clone()));

    // Run diagnostics
    let diagnostics = run_diagnostics(&s, &params.text_document.text);
    publish_diagnostics(sender, params.text_document.uri, diagnostics, Some(params.text_document.version));
}

fn handle_did_change(state: Arc<Mutex<ServerState>>, sender: Sender<Message>, params: DidChangeTextDocumentParams) {
    let mut s = state.lock().unwrap();
    // We requested Full sync, so content_changes[0].text is the full text
    if let Some(change) = params.content_changes.first() {
        s.documents.insert(params.text_document.uri.clone(), (params.text_document.version, change.text.clone()));

        let diagnostics = run_diagnostics(&s, &change.text);
        publish_diagnostics(sender, params.text_document.uri, diagnostics, Some(params.text_document.version));
    }
}

fn publish_diagnostics(sender: Sender<Message>, uri: Url, diagnostics: Vec<Diagnostic>, version: Option<i32>) {
    let params = PublishDiagnosticsParams {
        uri,
        diagnostics,
        version,
    };
    let not = Notification {
        method: PublishDiagnostics::METHOD.to_string(),
        params: serde_json::to_value(&params).unwrap(),
    };
    sender.send(Message::Notification(not)).unwrap();
}

fn run_diagnostics(state: &ServerState, text: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    // 1. Syntax Check via Parser
    let mut lexer = Lexer::new(text.to_string());
    match lexer.scan_tokens() {
        Ok(tokens) => {
            let mut parser = Parser::new(tokens);
            match parser.parse() {
                Ok(stmts) => {
                    // 2. Run Linter on AST
                    diagnostics.extend(state.linter.check(&stmts, text));
                }
                Err(e) => {
                     diagnostics.push(Diagnostic {
                        range: Range::default(),
                        severity: Some(DiagnosticSeverity::ERROR),
                        message: e.to_string(),
                        ..Default::default()
                    });
                }
            }
        }
        Err(e) => {
             diagnostics.push(Diagnostic {
                range: Range::default(),
                severity: Some(DiagnosticSeverity::ERROR),
                message: e.to_string(),
                ..Default::default()
            });
        }
    }

    diagnostics
}

fn handle_completion(state: Arc<Mutex<ServerState>>, params: CompletionParams) -> CompletionResponse {
    let s = state.lock().unwrap();
    let uri = params.text_document_position.text_document.uri;

    if let Some((_, text)) = s.documents.get(&uri) {
        let offset = position_to_offset(text, params.text_document_position.position);

        let core = eldritch_core::Interpreter::new();
        // core.load_builtins() is called in new()

        let (_start_idx, candidates) = core.complete(text, offset);

        let items: Vec<CompletionItem> = candidates.into_iter().map(|c| {
            CompletionItem {
                label: c,
                kind: Some(CompletionItemKind::FUNCTION),
                ..Default::default()
            }
        }).collect();

        return CompletionResponse::Array(items);
    }

    CompletionResponse::Array(vec![])
}

fn position_to_offset(text: &str, position: Position) -> usize {
    let mut offset = 0;
    for (i, line) in text.lines().enumerate() {
        if i == position.line as usize {
            offset += position.character as usize;
            return offset;
        }
        offset += line.len() + 1; // +1 for newline
    }
    if position.line as usize >= text.lines().count() {
        return text.len();
    }
    offset
}
