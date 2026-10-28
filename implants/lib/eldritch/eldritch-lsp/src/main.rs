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
        if has_errors {
            std::process::exit(1);
        }
        return;
    }

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(Backend::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}
