use anyhow::Result;
use crossbeam_channel::Sender;
use eldritch_core::{Lexer, Parser};
use eldritchv2::{Interpreter, Value};
use lsp_server::{Connection, Message, Notification, Response};
use lsp_types::{
    notification::{
        DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument,
        Notification as LspNotification, PublishDiagnostics,
    },
    request::{Completion, HoverRequest, Request as LspRequest},
    CompletionItem, CompletionItemKind, CompletionParams, CompletionResponse, Diagnostic,
    DiagnosticSeverity, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, Hover, HoverContents, HoverParams, InitializeParams, MarkupContent,
    MarkupKind, Position, PublishDiagnosticsParams, ServerCapabilities, TextDocumentSyncCapability,
    TextDocumentSyncKind, Url,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

mod linter;
use linter::{span_to_range, Linter};

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
        std::fs::File::create("/tmp/eldritch-lsp.log")
            .unwrap_or_else(|_| std::fs::File::create("eldritch-lsp.log").unwrap()),
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
        hover_provider: Some(lsp_types::HoverProviderCapability::Simple(true)),
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
                        sender
                            .send(Message::Response(Response {
                                id: req.id,
                                result: Some(result),
                                error: None,
                            }))
                            .unwrap();
                    }
                    HoverRequest::METHOD => {
                        let params: HoverParams = serde_json::from_value(req.params).unwrap();
                        let resp = handle_hover(state, params);
                        let result = serde_json::to_value(&resp).unwrap();
                        sender
                            .send(Message::Response(Response {
                                id: req.id,
                                result: Some(result),
                                error: None,
                            }))
                            .unwrap();
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
                        let params: DidOpenTextDocumentParams =
                            serde_json::from_value(not.params).unwrap();
                        handle_did_open(state, sender, params);
                    }
                    DidChangeTextDocument::METHOD => {
                        let params: DidChangeTextDocumentParams =
                            serde_json::from_value(not.params).unwrap();
                        handle_did_change(state, sender, params);
                    }
                    DidCloseTextDocument::METHOD => {
                        let params: DidCloseTextDocumentParams =
                            serde_json::from_value(not.params).unwrap();
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

fn handle_did_open(
    state: Arc<Mutex<ServerState>>,
    sender: Sender<Message>,
    params: DidOpenTextDocumentParams,
) {
    let mut s = state.lock().unwrap();
    s.documents.insert(
        params.text_document.uri.clone(),
        (
            params.text_document.version,
            params.text_document.text.clone(),
        ),
    );

    // Run diagnostics
    let diagnostics = run_diagnostics(&s, &params.text_document.text);
    publish_diagnostics(
        sender,
        params.text_document.uri,
        diagnostics,
        Some(params.text_document.version),
    );
}

fn handle_did_change(
    state: Arc<Mutex<ServerState>>,
    sender: Sender<Message>,
    params: DidChangeTextDocumentParams,
) {
    let mut s = state.lock().unwrap();
    // We requested Full sync, so content_changes[0].text is the full text
    if let Some(change) = params.content_changes.first() {
        s.documents.insert(
            params.text_document.uri.clone(),
            (params.text_document.version, change.text.clone()),
        );

        let diagnostics = run_diagnostics(&s, &change.text);
        publish_diagnostics(
            sender,
            params.text_document.uri,
            diagnostics,
            Some(params.text_document.version),
        );
    }
}

fn publish_diagnostics(
    sender: Sender<Message>,
    uri: Url,
    diagnostics: Vec<Diagnostic>,
    version: Option<i32>,
) {
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
                        range: span_to_range(e.span, text),
                        severity: Some(DiagnosticSeverity::ERROR),
                        message: e.message,
                        ..Default::default()
                    });
                }
            }
        }
        Err(e) => {
            diagnostics.push(Diagnostic {
                range: span_to_range(e.span, text),
                severity: Some(DiagnosticSeverity::ERROR),
                message: e.message,
                ..Default::default()
            });
        }
    }

    diagnostics
}

fn handle_completion(
    state: Arc<Mutex<ServerState>>,
    params: CompletionParams,
) -> CompletionResponse {
    let s = state.lock().unwrap();
    let uri = params.text_document_position.text_document.uri;

    if let Some((_, text)) = s.documents.get(&uri) {
        let offset = position_to_offset(text, params.text_document_position.position);

        // Use facade interpreter with all libraries loaded (including fake agent for completion)
        let interp = Interpreter::new().with_default_libs().with_fake_agent();

        let (_start_idx, candidates) = interp.complete(text, offset);

        let items: Vec<CompletionItem> = candidates
            .into_iter()
            .map(|c| {
                // Determine Kind
                let kind = if let Some(val) = interp.lookup_variable(&c) {
                    match val {
                        Value::Foreign(_) => Some(CompletionItemKind::MODULE), // Libraries
                        Value::NativeFunction(_, _)
                        | Value::NativeFunctionWithKwargs(_, _)
                        | Value::Function(_) => Some(CompletionItemKind::FUNCTION), // Builtins
                        _ => Some(CompletionItemKind::VARIABLE),
                    }
                } else {
                    // If resolving fails, it's likely a method or property from dot-completion
                    Some(CompletionItemKind::METHOD)
                };

                CompletionItem {
                    label: c,
                    kind,
                    ..Default::default()
                }
            })
            .collect();

        return CompletionResponse::Array(items);
    }

    CompletionResponse::Array(vec![])
}

fn handle_hover(state: Arc<Mutex<ServerState>>, params: HoverParams) -> Option<Hover> {
    let s = state.lock().unwrap();
    let uri = params.text_document_position_params.text_document.uri;

    if let Some((_, text)) = s.documents.get(&uri) {
        let offset = position_to_offset(text, params.text_document_position_params.position);

        // We need to resolve the symbol under cursor.
        // We can reuse the `complete` logic logic partially, but we want the full token chain.
        // e.g. "file.append" -> we want to resolve "file", then check "append" on it.
        // Or simple variable lookup.

        let interp = Interpreter::new().with_default_libs().with_fake_agent();

        let (start, end) = find_word_bounds(text, offset);
        let _word = &text[start..end];

        // For now, let's look backwards for dotted chain.
        let full_expr = expand_dotted_chain(text, start, end);

        // Try to evaluate the parent object if there is a dot
        if let Some((obj_name, method_name)) = full_expr.rsplit_once('.') {
            // Resolve obj_name
            if let Some(val) = interp.lookup_variable(obj_name) {
                // Check if it's a library or built-in
                if let Some(sig) = get_method_signature(&val, method_name) {
                    return Some(Hover {
                        contents: HoverContents::Markup(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: format_signature(&sig),
                        }),
                        range: None,
                    });
                }
            }
        } else {
            // It's a global variable or function
            if let Some(val) = interp.lookup_variable(&full_expr) {
                // Should show type or doc if available
                // Currently variables don't carry docs, but libraries (ForeignValue) do?
                // Actually ForeignValue is an object.
                // If it's a ForeignValue (library module), we can describe it.
                let desc = match val {
                    Value::Foreign(f) => format!("Module: {}", f.type_name()),
                    Value::NativeFunction(name, _) | Value::NativeFunctionWithKwargs(name, _) => {
                        format!("Built-in function: {}", name)
                    }
                    Value::Function(f) => format!("Function: {}", f.name),
                    _ => format!("Variable: {}", full_expr),
                };
                return Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: desc,
                    }),
                    range: None,
                });
            }
        }
    }
    None
}

// Helpers

fn find_word_bounds(text: &str, offset: usize) -> (usize, usize) {
    if offset > text.len() {
        return (0, 0);
    }

    // Convert byte offset to char index for safe iteration
    // Actually, iterating chars is easier.

    // Find "start" by scanning backwards from offset
    // This requires char indices.
    let char_indices: Vec<(usize, char)> = text.char_indices().collect();

    // Find index in char_indices that corresponds to offset
    // or the one immediately before it if offset is between chars (not likely if valid utf8 split, but cursor can be anywhere)
    // Actually, LSP offset is typically UTF-16 code units. But here we have UTF-8 text.
    // Assuming `offset` passed in `handle_hover` is byte offset calculated by `position_to_offset`.

    let mut current_idx = 0;
    for (i, (byte_idx, _)) in char_indices.iter().enumerate() {
        if *byte_idx >= offset {
            current_idx = i;
            break;
        }
        current_idx = i + 1; // if at end
    }
    if current_idx >= char_indices.len() {
        current_idx = char_indices.len();
    }

    // Scan backwards
    let mut start_char_idx = current_idx;
    while start_char_idx > 0 {
        let (_, c) = char_indices[start_char_idx - 1];
        if !c.is_alphanumeric() && c != '_' {
            break;
        }
        start_char_idx -= 1;
    }

    // Scan forwards
    let mut end_char_idx = current_idx;
    while end_char_idx < char_indices.len() {
        let (_, c) = char_indices[end_char_idx];
        if !c.is_alphanumeric() && c != '_' {
            break;
        }
        end_char_idx += 1;
    }

    let start_byte = if start_char_idx < char_indices.len() {
        char_indices[start_char_idx].0
    } else {
        text.len()
    };

    let end_byte = if end_char_idx < char_indices.len() {
        char_indices[end_char_idx].0
    } else {
        text.len()
    };

    (start_byte, end_byte)
}

fn expand_dotted_chain(text: &str, start_byte: usize, end_byte: usize) -> String {
    let char_indices: Vec<(usize, char)> = text.char_indices().collect();

    // Find char index for start_byte
    let mut current_char_idx = 0;
    for (i, (b, _)) in char_indices.iter().enumerate() {
        if *b == start_byte {
            current_char_idx = i;
            break;
        }
    }

    let mut s = current_char_idx;

    // Look backwards
    while s > 0 {
        let (_, c) = char_indices[s - 1];
        if c == '.' {
            s -= 1;
            // Now verify identifier
            let id_end = s;
            while s > 0 {
                let (_, c2) = char_indices[s - 1];
                if !c2.is_alphanumeric() && c2 != '_' {
                    break;
                }
                s -= 1;
            }
            if s == id_end {
                // Dot without identifier
                s += 1;
                break;
            }
        } else {
            break;
        }
    }

    let effective_start_byte = char_indices[s].0;
    text[effective_start_byte..end_byte].to_string()
}

fn get_method_signature(val: &Value, method: &str) -> Option<eldritch_core::MethodSignature> {
    match val {
        Value::Foreign(f) => f.get_method_signature(method),
        _ => eldritch_core::get_native_method_signature(val, method),
    }
}

fn format_signature(sig: &eldritch_core::MethodSignature) -> String {
    let mut s = format!("`{}(", sig.name);
    for (i, p) in sig.params.iter().enumerate() {
        if i > 0 {
            s.push_str(", ");
        }
        s.push_str(&p.name);
        if let Some(t) = &p.type_name {
            s.push_str(": ");
            s.push_str(t);
        }
    }
    s.push_str(")`");

    if let Some(ret) = &sig.return_type {
        s.push_str(" -> `");
        s.push_str(ret);
        s.push('`');
    }

    if let Some(doc) = &sig.doc {
        s.push_str("\n\n");
        s.push_str(doc);
    }

    s
}

fn position_to_offset(text: &str, position: Position) -> usize {
    let mut offset = 0;
    let target_line = position.line as usize;
    let target_char = position.character as usize;

    for (i, line) in text.lines().enumerate() {
        if i == target_line {
            // Count characters to target_char
            // LSP uses UTF-16 code units for character offset
            // We need to walk the string and count UTF-16 units
            let mut utf16_count = 0;
            let mut byte_count = 0;
            for c in line.chars() {
                if utf16_count >= target_char {
                    break;
                }
                utf16_count += c.len_utf16();
                byte_count += c.len_utf8();
            }
            offset += byte_count;
            return offset;
        }
        // Add line length + newline chars
        offset += line.len();

        // Handle newline characters that `lines()` stripped
        // We need to look at original text to see if it was \n or \r\n
        // But iterating lines() loses that info.
        // Better way: manual iteration over text bytes/chars looking for newlines.
    }

    // Re-implementation using direct char iteration to be safe and accurate with newlines
    let mut line = 0;
    let mut utf16_col = 0;
    let mut byte_offset = 0;

    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if line == target_line && utf16_col == target_char {
            return byte_offset;
        }

        byte_offset += c.len_utf8();

        if c == '\n' {
            line += 1;
            utf16_col = 0;
        } else if c == '\r' {
            // Check for \r\n
            if let Some(&'\n') = chars.peek() {
                // Next is \n, consume it in next iteration
                // line increment happens on \n
            } else {
                // Just \r (classic mac or weirdness)
                line += 1;
                utf16_col = 0;
            }
        } else if line == target_line {
            utf16_col += c.len_utf16();
        }
    }

    byte_offset
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completion_string_method() {
        let code = r#"
def get_env():
    envs = sys.get_env()
    for key, value in envs.items():
        print(f"{key}={value}")
    "".s"#;

        // Offset of the end of the string
        let offset = code.len();

        // Use facade
        let interp = Interpreter::new().with_default_libs();
        let (_start, candidates) = interp.complete(code, offset);

        // We expect string methods starting with 's'
        assert!(candidates.contains(&"split".to_string()));
        assert!(candidates.contains(&"strip".to_string()));

        // We expect NO globals like "set" or "sorted"
        assert!(!candidates.contains(&"set".to_string()));
        assert!(!candidates.contains(&"sorted".to_string()));
    }

    #[test]
    fn test_completion_libraries_and_literals() {
        // Test library "agent" presence
        let code1 = "ag";
        let interp = Interpreter::new().with_default_libs().with_fake_agent();
        let (_, candidates1) = interp.complete(code1, code1.len());
        assert!(candidates1.contains(&"agent".to_string()));

        // Test library method "agent.get_config"
        let code2 = "agent.";
        let (_, candidates2) = interp.complete(code2, code2.len());
        assert!(!candidates2.is_empty());

        // Test literal list completion "[]."
        let code3 = "[].";
        let (_, candidates3) = interp.complete(code3, code3.len());
        assert!(candidates3.contains(&"append".to_string()));
        assert!(candidates3.contains(&"sort".to_string()));

        // Test literal dict completion "{}."
        let code4 = "{}.";
        let (_, candidates4) = interp.complete(code4, code4.len());
        assert!(candidates4.contains(&"keys".to_string()));
        assert!(candidates4.contains(&"get".to_string()));
    }

    #[test]
    fn test_hover_list_append() {
        let _code = "file.list";
        let _offset = 7; // end

        // We can manually call helper logic
        let interp = Interpreter::new().with_default_libs();
        let val = interp
            .lookup_variable("file")
            .expect("file lib should be present");
        let sig = get_method_signature(&val, "list");

        assert!(sig.is_some());
        let s = sig.unwrap();
        assert_eq!(s.name, "list");
    }
}
