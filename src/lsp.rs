//! Language Server Protocol implementation for Glimmer-Weave
//!
//! This module provides an LSP server that enables IDE features like:
//! - Syntax error diagnostics
//! - Type checking and semantic errors
//! - Hover information (type hints)
//! - Go-to-definition
//! - Autocomplete
//! - Document symbols

#[cfg(feature = "lsp")]
use std::collections::HashMap;
#[cfg(feature = "lsp")]
use std::sync::Arc;

#[cfg(feature = "lsp")]
use tokio::sync::RwLock;
#[cfg(feature = "lsp")]
use tower_lsp::jsonrpc::Result as JsonRpcResult;
#[cfg(feature = "lsp")]
use tower_lsp::lsp_types::*;
#[cfg(feature = "lsp")]
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[cfg(feature = "lsp")]
use crate::lexer::Lexer;
#[cfg(feature = "lsp")]
use crate::parser::Parser;
#[cfg(feature = "lsp")]
use crate::semantic::SemanticAnalyzer;
#[cfg(feature = "lsp")]
use crate::type_inference::TypeInference;
#[cfg(feature = "lsp")]
use crate::symbol_table::{SymbolCollector, SymbolTable};

/// Document state stored in the LSP server
#[cfg(feature = "lsp")]
#[derive(Debug, Clone)]
struct Document {
    /// The document's URI
    #[allow(dead_code)]
    uri: Url,
    /// The document's text content
    text: String,
    /// Version number for synchronization
    #[allow(dead_code)]
    version: i32,
    /// Symbol table for go-to-definition and other features
    symbols: SymbolTable,
}

/// Backend state for the Glimmer-Weave LSP server
#[cfg(feature = "lsp")]
pub struct GlimmerWeaveBackend {
    /// LSP client for sending notifications/requests to the IDE
    client: Client,
    /// In-memory document store
    documents: Arc<RwLock<HashMap<Url, Document>>>,
}

#[cfg(feature = "lsp")]
impl GlimmerWeaveBackend {
    /// Create a new LSP backend
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Parse a document and return diagnostics and symbol table
    async fn analyze_document(&self, _uri: &Url, text: &str) -> (Vec<Diagnostic>, SymbolTable) {
        let mut diagnostics = Vec::new();
        let empty_symbols = SymbolTable::new();

        // Lexical analysis
        let mut lexer = Lexer::new(text);
        let tokens = lexer.tokenize_positioned();

        // Syntactic analysis
        let mut parser = Parser::new(tokens);
        let ast = match parser.parse() {
            Ok(ast) => ast,
            Err(e) => {
                // Parser error
                let position = e.position;
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position {
                            line: position as u32,
                            character: 0,
                        },
                        end: Position {
                            line: position as u32,
                            character: 0,
                        },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: None,
                    code_description: None,
                    source: Some("glimmer-weave-parser".to_string()),
                    message: format!("Parse error: {}", e.message),
                    related_information: None,
                    tags: None,
                    data: None,
                });
                return (diagnostics, empty_symbols);
            }
        };

        // Build symbol table
        let collector = SymbolCollector::new();
        let symbols = collector.collect(&ast);

        // Semantic analysis
        let mut analyzer = SemanticAnalyzer::new();
        if let Err(errors) = analyzer.analyze(&ast) {
            for error in errors {
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position {
                            line: 0,
                            character: 0,
                        },
                        end: Position {
                            line: 0,
                            character: 0,
                        },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: None,
                    code_description: None,
                    source: Some("glimmer-weave-semantic".to_string()),
                    message: format!("{:?}", error),
                    related_information: None,
                    tags: None,
                    data: None,
                });
            }
        }

        // Type inference
        let mut type_engine = TypeInference::new();
        if let Err(error) = type_engine.infer_program(&ast) {
            diagnostics.push(Diagnostic {
                range: Range {
                    start: Position {
                        line: 0,
                        character: 0,
                    },
                    end: Position {
                        line: 0,
                        character: 0,
                    },
                },
                severity: Some(DiagnosticSeverity::ERROR),
                code: None,
                code_description: None,
                source: Some("glimmer-weave-types".to_string()),
                message: error,
                related_information: None,
                tags: None,
                data: None,
            });
        }

        (diagnostics, symbols)
    }

    /// Send diagnostics to the client
    async fn publish_diagnostics(&self, uri: Url, diagnostics: Vec<Diagnostic>) {
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }

    /// Extract the word at a given position in the text
    fn get_word_at_position(&self, text: &str, line: usize, character: usize) -> Option<String> {
        let lines: Vec<&str> = text.lines().collect();
        if line >= lines.len() {
            return None;
        }

        let line_text = lines[line];
        if character >= line_text.len() {
            return None;
        }

        // Find word boundaries
        let start = line_text[..character]
            .rfind(|c: char| !c.is_alphanumeric() && c != '_')
            .map(|i| i + 1)
            .unwrap_or(0);

        let end = line_text[character..]
            .find(|c: char| !c.is_alphanumeric() && c != '_')
            .map(|i| character + i)
            .unwrap_or(line_text.len());

        if start >= end {
            return None;
        }

        Some(line_text[start..end].to_string())
    }
}

#[cfg(feature = "lsp")]
#[tower_lsp::async_trait]
impl LanguageServer for GlimmerWeaveBackend {
    async fn initialize(&self, _: InitializeParams) -> JsonRpcResult<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "glimmer-weave-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string(), " ".to_string()]),
                    work_done_progress_options: WorkDoneProgressOptions::default(),
                    all_commit_characters: None,
                    completion_item: None,
                }),
                definition_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                workspace: Some(WorkspaceServerCapabilities {
                    workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                        supported: Some(true),
                        change_notifications: Some(OneOf::Left(true)),
                    }),
                    file_operations: None,
                }),
                ..ServerCapabilities::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Glimmer-Weave LSP server initialized")
            .await;
    }

    async fn shutdown(&self) -> JsonRpcResult<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        let version = params.text_document.version;

        // Analyze and build symbol table
        let (diagnostics, symbols) = self.analyze_document(&uri, &text).await;

        // Store document with symbol table
        {
            let mut documents = self.documents.write().await;
            documents.insert(
                uri.clone(),
                Document {
                    uri: uri.clone(),
                    text: text.clone(),
                    version,
                    symbols,
                },
            );
        }

        // Send diagnostics
        self.publish_diagnostics(uri, diagnostics).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let version = params.text_document.version;

        // Get full document text (we use FULL sync)
        if let Some(change) = params.content_changes.first() {
            let text = change.text.clone();

            // Analyze and build symbol table
            let (diagnostics, symbols) = self.analyze_document(&uri, &text).await;

            // Update document with symbol table
            {
                let mut documents = self.documents.write().await;
                documents.insert(
                    uri.clone(),
                    Document {
                        uri: uri.clone(),
                        text: text.clone(),
                        version,
                        symbols,
                    },
                );
            }

            // Send diagnostics
            self.publish_diagnostics(uri, diagnostics).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;

        // Remove document from store
        {
            let mut documents = self.documents.write().await;
            documents.remove(&uri);
        }

        // Clear diagnostics
        self.publish_diagnostics(uri, Vec::new()).await;
    }

    async fn hover(&self, params: HoverParams) -> JsonRpcResult<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let _position = params.text_document_position_params.position;

        // Get document
        let text = {
            let documents = self.documents.read().await;
            documents.get(&uri).map(|doc| doc.text.clone())
        };

        if let Some(_text) = text {
            // TODO: Implement hover information
            // For now, return a placeholder
            Ok(Some(Hover {
                contents: HoverContents::Scalar(MarkedString::String(
                    "Glimmer-Weave hover information coming soon!".to_string(),
                )),
                range: None,
            }))
        } else {
            Ok(None)
        }
    }

    async fn completion(&self, params: CompletionParams) -> JsonRpcResult<Option<CompletionResponse>> {
        let _uri = params.text_document_position.text_document.uri;
        let _position = params.text_document_position.position;

        // TODO: Implement intelligent completion
        // For now, return basic keywords
        let keywords = vec![
            "bind", "weave", "set", "chant", "yield", "should", "then", "otherwise",
            "end", "for", "each", "in", "whilst", "attempt", "harmonize", "match",
            "when", "form", "with", "as", "Triumph", "Mishap", "Present", "Absent",
            "borrow", "mut", "request",
        ];

        let items: Vec<CompletionItem> = keywords
            .iter()
            .map(|keyword| CompletionItem {
                label: keyword.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some(format!("Glimmer-Weave keyword: {}", keyword)),
                ..Default::default()
            })
            .collect();

        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> JsonRpcResult<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        // Get document and symbol table
        let doc = {
            let documents = self.documents.read().await;
            documents.get(&uri).cloned()
        };

        let doc = match doc {
            Some(d) => d,
            None => return Ok(None),
        };

        // Extract the word at the cursor position
        let word = self.get_word_at_position(&doc.text, position.line as usize, position.character as usize);
        let word = match word {
            Some(w) => w,
            None => return Ok(None),
        };

        // Look up the symbol in the symbol table
        let symbol = doc.symbols.find_at_position(&word, position.line as usize + 1, position.character as usize + 1);
        let symbol = match symbol {
            Some(s) => s,
            None => return Ok(None),
        };

        // Convert source location to LSP Location
        if symbol.definition_span.start.is_known() {
            let location = Location {
                uri: uri.clone(),
                range: Range {
                    start: Position {
                        line: (symbol.definition_span.start.line.saturating_sub(1)) as u32,
                        character: (symbol.definition_span.start.column.saturating_sub(1)) as u32,
                    },
                    end: Position {
                        line: (symbol.definition_span.end.line.saturating_sub(1)) as u32,
                        character: (symbol.definition_span.end.column.saturating_sub(1)) as u32,
                    },
                },
            };
            Ok(Some(GotoDefinitionResponse::Scalar(location)))
        } else {
            Ok(None)
        }
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> JsonRpcResult<Option<DocumentSymbolResponse>> {
        let _uri = params.text_document.uri;

        // TODO: Implement document symbols
        // This requires extracting function/struct definitions from AST
        Ok(None)
    }
}

/// Create and run the LSP server
#[cfg(feature = "lsp")]
pub async fn run_server() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(GlimmerWeaveBackend::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}
