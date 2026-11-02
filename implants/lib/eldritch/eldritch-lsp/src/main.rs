use eldritch_lsp::Backend;
use eldritch_lsp::analysis::{TomeMetadata, analyze_document};
use eldritch_lsp::linter::LintRegistry;
use eldritch_lsp::stdlib::StdlibIndex;
use std::path::PathBuf;
use tower_lsp::lsp_types::DiagnosticSeverity;
use tower_lsp::{LspService, Server};

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("eldritch-lsp {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    if let Some(check_idx) = args.iter().position(|a| a == "--check") {
        let files = &args[check_idx + 1..];
        let stdlib = StdlibIndex::new();
        let linter = LintRegistry::default();
        let mut has_errors = false;

        for file_arg in files {
            if file_arg.starts_with('-') {
                continue;
            }
            let path = PathBuf::from(file_arg);
            match std::fs::read_to_string(&path) {
                Ok(source) => {
                    let meta = TomeMetadata::load_for_document(&path);
                    let result = analyze_document(&source, &stdlib, &linter, meta);
                    for diag in &result.diagnostics {
                        let sev = match diag.severity {
                            Some(DiagnosticSeverity::ERROR) => {
                                has_errors = true;
                                "ERROR"
                            }
                            Some(DiagnosticSeverity::WARNING) => "WARN",
                            _ => "INFO",
                        };
                        println!(
                            "{}:{}:{}: [{}] {}",
                            path.display(),
                            diag.range.start.line + 1,
                            diag.range.start.character + 1,
                            sev,
                            diag.message
                        );
                    }
                }
                Err(err) => {
                    eprintln!("Failed to read {}: {}", path.display(), err);
                    has_errors = true;
                }
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
            "def", "if", "elif", "else", "for", "while", "return", "try", "except", "finally",
            "as", "raise", "import", "true", "false", "none",
        ];
        for kw in keywords {
            items.push(CompletionItem {
                label: kw.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                ..Default::default()
            });
        }
        return;
    }

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(Backend::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}
