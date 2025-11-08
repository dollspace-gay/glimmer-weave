//! Glimmer-Weave Language Server Protocol (LSP) binary
//!
//! This binary starts the Glimmer-Weave LSP server, which provides IDE features
//! like autocomplete, go-to-definition, error checking, and more.
//!
//! # Usage
//!
//! ```bash
//! glimmer-lsp
//! ```
//!
//! The server communicates over stdin/stdout using the JSON-RPC protocol.

use glimmer_weave::lsp;

#[tokio::main]
async fn main() {
    // Start the LSP server
    lsp::run_server().await;
}
