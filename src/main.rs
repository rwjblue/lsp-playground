use std::collections::HashMap;
use tokio::sync::RwLock as AsyncRwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
struct Backend {
    client: Client,
    documents: AsyncRwLock<HashMap<Url, String>>,
}

impl Backend {
    fn new(client: Client) -> Self {
        Backend {
            client,
            documents: AsyncRwLock::new(HashMap::new()),
        }
    }

    async fn send_diagnostics(&self, uri: &Url) {
        let diagnostics = if let Some(text) = self.documents.read().await.get(uri) {
            text.lines()
                .enumerate()
                .filter_map(|(line_number, line)| {
                    if line.contains("TODO") {
                        Some(Diagnostic {
                            range: Range {
                                start: Position {
                                    line: line_number as u32,
                                    character: 0,
                                },
                                end: Position {
                                    line: line_number as u32,
                                    character: line.len() as u32,
                                },
                            },
                            severity: Some(DiagnosticSeverity::WARNING),
                            code: None,
                            code_description: None,
                            source: Some("todo-checker".to_string()),
                            message: "TODO found".to_string(),
                            related_information: None,
                            tags: None,
                            data: None,
                        })
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        } else {
            vec![]
        };

        self.client
            .log_message(
                MessageType::LOG,
                format!("sending diagnostics: {:?}", diagnostics),
            )
            .await;

        self.client
            .publish_diagnostics(uri.clone(), diagnostics, None)
            .await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                    DiagnosticOptions {
                        inter_file_dependencies: false,
                        workspace_diagnostics: false,
                        ..DiagnosticOptions::default()
                    },
                )),
                ..ServerCapabilities::default()
            },
            ..InitializeResult::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::LOG, "lsp_playground server initialized!")
            .await;
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;

        self.client
            .log_message(MessageType::LOG, format!("did_open: {}", uri))
            .await;

        self.documents.write().await.insert(uri, text);
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = &params.text_document.uri;
        let changes = &params.content_changes;

        self.client
            .log_message(MessageType::LOG, format!("did_change: {}", uri))
            .await;

        if let Some(change) = changes.first() {
            self.documents
                .write()
                .await
                // TODO: figure out if we can avoid cloning here
                .insert(uri.clone(), change.text.clone());
        }

        self.send_diagnostics(uri).await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(Backend::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}
