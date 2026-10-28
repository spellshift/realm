pub mod analysis;
pub mod linter;
pub mod stdlib;

use analysis::{
    AnalysisResult, TomeMetadata, analyze_document, complete_at, document_symbols, goto_definition,
    hover_at, signature_help_at,
};
use linter::LintRegistry;
use std::collections::HashMap;
use std::sync::Arc;
use stdlib::StdlibIndex;
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

pub struct Backend {
    pub client: Client,
    pub linter: LintRegistry,
    pub stdlib: Arc<StdlibIndex>,
    pub documents: Arc<RwLock<HashMap<Url, AnalysisResult>>>,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            linter: LintRegistry::default(),
            stdlib: Arc::new(StdlibIndex::new()),
            documents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn analyze_and_publish(&self, uri: Url, text: &str, version: Option<i32>) {
        let metadata = uri
            .to_file_path()
            .ok()
            .and_then(|p| TomeMetadata::load_for_document(&p));
        let result = analyze_document(text, &self.stdlib, &self.linter, metadata);
        let diagnostics = result.diagnostics.clone();
        self.documents.write().await.insert(uri.clone(), result);
        self.client
            .publish_diagnostics(uri, diagnostics, version)
            .await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _params: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "eldritch-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::FULL),
                        save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
                            include_text: Some(true),
                        })),
                        ..Default::default()
                    },
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![
                        ".".to_string(),
                        "'".to_string(),
                        "\"".to_string(),
                        "(".to_string(),
                    ]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                signature_help_provider: Some(SignatureHelpOptions {
                    trigger_characters: Some(vec!["(".to_string(), ",".to_string()]),
                    retrigger_characters: Some(vec![",".to_string()]),
                    work_done_progress_options: WorkDoneProgressOptions::default(),
                }),
                document_symbol_provider: Some(OneOf::Left(true)),
                definition_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Eldritch Language Server initialized.")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let doc = params.text_document;
        self.analyze_and_publish(doc.uri, &doc.text, Some(doc.version))
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let version = Some(params.text_document.version);
        if let Some(change) = params.content_changes.into_iter().last() {
            self.analyze_and_publish(uri, &change.text, version).await;
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        if let Some(text) = params.text {
            self.analyze_and_publish(params.text_document.uri, &text, None)
                .await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        self.documents.write().await.remove(&uri);
        self.client.publish_diagnostics(uri, Vec::new(), None).await;
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;
        let pos = params.text_document_position.position;
        let docs = self.documents.read().await;
        if let Some(analysis) = docs.get(uri) {
            let items = complete_at(analysis, pos, &self.stdlib);
            return Ok(Some(CompletionResponse::Array(items)));
        }
        Ok(None)
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;
        let docs = self.documents.read().await;
        if let Some(analysis) = docs.get(uri) {
            return Ok(hover_at(analysis, pos, &self.stdlib));
        }
        Ok(None)
    }

    async fn signature_help(&self, params: SignatureHelpParams) -> Result<Option<SignatureHelp>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;
        let docs = self.documents.read().await;
        if let Some(analysis) = docs.get(uri) {
            return Ok(signature_help_at(analysis, pos, &self.stdlib));
        }
        Ok(None)
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = &params.text_document.uri;
        let docs = self.documents.read().await;
        if let Some(analysis) = docs.get(uri) {
            return Ok(Some(DocumentSymbolResponse::Nested(document_symbols(
                analysis,
            ))));
        }
        Ok(None)
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;
        let docs = self.documents.read().await;
        if let Some(analysis) = docs.get(uri) {
            return Ok(goto_definition(analysis, pos, uri));
        }
        Ok(None)
    }
}
