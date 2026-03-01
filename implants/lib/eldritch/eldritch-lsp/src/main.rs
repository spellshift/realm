use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

mod linter;
mod stdlib;

use linter::LintRegistry;
use stdlib::StdlibIndex;
use eldritch_core::{Parser, Lexer};

struct Backend {
    client: Client,
    /// Registry of lint rules.
    linter: LintRegistry,
    /// Index of the standard library.
    stdlib: Arc<RwLock<StdlibIndex>>,
    /// In-memory cache of document contents.
    documents: Arc<RwLock<HashMap<Url, String>>>,
}

impl Backend {
    async fn validate_document(&self, uri: Url, text: &str, version: Option<i32>) {
        let mut diagnostics = Vec::new();

        // 1. Parsing Phase (using eldritch-core)
        let mut lexer = Lexer::new(text.to_string());
        let tokens = lexer.scan_tokens();

        let mut parser = Parser::new(tokens);
        let (ast_stmts, parse_errors) = parser.parse();

        // 2. Syntax Error Diagnostics
        for err in parse_errors {
            // Convert Span to LSP Range
            // eldritch_core::token::Span has {start, end, line}.
            // This is slightly tricky because Span in `token.rs` only has `line` and byte indices.
            // LSP needs line/character.
            // Simplified mapping: we trust the `line` from Span, but we need to calculate `character`.
            // Ideally, we'd use a line indexer. For "Best Effort", we can estimate or just highlight the line.

            // Note: `line` in Span is 1-based. LSP is 0-based.
            let line_idx = err.span.line.saturating_sub(1) as u32;

            let range = Range {
                start: Position { line: line_idx, character: 0 },
                end: Position { line: line_idx, character: u32::MAX }, // Highlight full line if column unknown
            };

            diagnostics.push(Diagnostic {
                range,
                severity: Some(DiagnosticSeverity::ERROR),
                code: Some(NumberOrString::String("syntax-error".to_string())),
                source: Some("eldritch-parser".to_string()),
                message: err.message,
                ..Default::default()
            });
        }

        // 3. Linting Phase (Best Effort)
        // Pass the AST if parsing succeeded partially (ast_stmts might contain valid stmts even with errors).
        let lint_diags = self.linter.run(Some(&ast_stmts), text);
        diagnostics.extend(lint_diags);

        // 4. Publish Diagnostics
        self.client.publish_diagnostics(uri, diagnostics, version).await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        let root_uri = params.root_uri.or_else(|| {
            params.root_path.map(|path| Url::from_file_path(path).unwrap())
        });

        if let Some(uri) = root_uri {
            if let Ok(path) = uri.to_file_path() {
                // Heuristic: stdlib is at implants/lib/eldritch/stdlib
                // We assume the workspace root is the repo root.
                // Or we look for the stdlib relative to workspace.
                let stdlib_path = path.join("implants/lib/eldritch/stdlib");

                let stdlib_ref = self.stdlib.clone();
                tokio::spawn(async move {
                    let mut index = stdlib_ref.write().await;
                    index.scan(stdlib_path);
                });
            }
        }

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![".".to_string(), ":".to_string()]),
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Eldritch LSP initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let text = params.text_document.text.clone();
        let version = Some(params.text_document.version);

        self.documents.write().await.insert(uri.clone(), text.clone());
        self.validate_document(uri, &text, version).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let version = Some(params.text_document.version);
        // We use Full sync, so the first change event has the full text
        if let Some(change) = params.content_changes.into_iter().next() {
            self.documents.write().await.insert(uri.clone(), change.text.clone());
            self.validate_document(uri, &change.text, version).await;
        }
    }

    async fn completion(&self, _params: CompletionParams) -> Result<Option<CompletionResponse>> {
        // Context-aware completion logic
        // For now, we mix in:
        // 1. Stdlib modules
        // 2. Keywords (Safe Mode fallback)

        let stdlib = self.stdlib.read().await;
        let mut items = Vec::new();

        // Add Stdlib modules
        for module in stdlib.get_completions() {
            items.push(CompletionItem {
                label: module,
                kind: Some(CompletionItemKind::MODULE),
                detail: Some("Standard Library".to_string()),
                ..Default::default()
            });
        }

        // Add Keywords
        let keywords = vec![
            "def", "if", "else", "for", "while", "return", "import", "true", "false", "none"
        ];
        for kw in keywords {
            items.push(CompletionItem {
                label: kw.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                ..Default::default()
            });
        }

        Ok(Some(CompletionResponse::Array(items)))
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend {
        client,
        linter: LintRegistry::default(),
        stdlib: Arc::new(RwLock::new(StdlibIndex::new())),
        documents: Arc::new(RwLock::new(HashMap::new())),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
