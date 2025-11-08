# Glimmer-Weave Language Server Protocol (LSP)

The Glimmer-Weave LSP server provides IDE features like autocomplete, diagnostics, hover information, and more.

## Features

### Currently Implemented

- ✅ **Document Synchronization** - Tracks open/edited/closed files
- ✅ **Diagnostics** - Real-time error reporting from:
  - Lexer (tokenization errors)
  - Parser (syntax errors)
  - Semantic analyzer (type errors, undefined variables, etc.)
  - Type inference engine (type mismatches)
- ✅ **Hover Information** - Show information when hovering over code (basic implementation)
- ✅ **Autocomplete** - Keyword suggestions for Glimmer-Weave syntax
- ✅ **Go-to-Definition** - Jump to variable/function/struct definitions
  - Supports `bind`, `weave`, `chant`, and `form` definitions
  - Automatically builds symbol table during parsing
  - Handles shadowing and scopes correctly
- ✅ **Document Symbols** - Outline view of functions and structs (placeholder)

### Planned Features

- ⚠️ **Find References** - Find all uses of a symbol
- ⚠️ **Rename** - Rename symbols across files
- ⚠️ **Code Actions** - Quick fixes and refactorings
- ⚠️ **Formatting** - Auto-format code
- ⚠️ **Semantic Highlighting** - Better syntax highlighting based on semantics

## Building

Build the LSP server with the `lsp` feature:

```bash
cargo build --bin glimmer-lsp --features lsp --release
```

The binary will be located at:
- Windows: `target\release\glimmer-lsp.exe`
- Linux/macOS: `target/release/glimmer-lsp`

## Usage

### VS Code

1. **Build the LSP server** (see above)

2. **Create a VS Code extension** (or use the configuration below for testing)

3. **Add to your VS Code `settings.json`:**

```json
{
  "glimmer-weave.server.path": "/path/to/glimmer-lsp",
  "glimmer-weave.trace.server": "verbose"
}
```

4. **Create a minimal VS Code extension** (see `vscode-extension/` directory)

### Other Editors

The LSP server communicates over stdin/stdout using JSON-RPC, so it should work with any LSP-compatible editor:

- **Neovim**: Use [nvim-lspconfig](https://github.com/neovim/nvim-lspconfig)
- **Emacs**: Use [lsp-mode](https://github.com/emacs-lsp/lsp-mode)
- **Sublime Text**: Use [LSP](https://github.com/sublimelsp/LSP)
- **Vim**: Use [vim-lsp](https://github.com/prabirshrestha/vim-lsp)

## VS Code Extension Setup

### Quick Start (Development)

1. **Create the extension directory:**

```bash
mkdir -p vscode-extension
cd vscode-extension
npm init -y
npm install --save vscode-languageclient
```

2. **Create `package.json`:**

```json
{
  "name": "glimmer-weave-vscode",
  "displayName": "Glimmer-Weave Language Support",
  "description": "Language support for Glimmer-Weave",
  "version": "0.1.0",
  "publisher": "glimmer-weave",
  "engines": {
    "vscode": "^1.80.0"
  },
  "categories": ["Programming Languages"],
  "activationEvents": ["onLanguage:glimmer-weave"],
  "main": "./out/extension.js",
  "contributes": {
    "languages": [{
      "id": "glimmer-weave",
      "aliases": ["Glimmer-Weave", "glimmer-weave"],
      "extensions": [".gw"],
      "configuration": "./language-configuration.json"
    }],
    "grammars": [{
      "language": "glimmer-weave",
      "scopeName": "source.glimmer-weave",
      "path": "./syntaxes/glimmer-weave.tmLanguage.json"
    }],
    "configuration": {
      "type": "object",
      "title": "Glimmer-Weave",
      "properties": {
        "glimmer-weave.server.path": {
          "type": "string",
          "default": "glimmer-lsp",
          "description": "Path to the Glimmer-Weave LSP server executable"
        },
        "glimmer-weave.trace.server": {
          "type": "string",
          "enum": ["off", "messages", "verbose"],
          "default": "off",
          "description": "Traces the communication between VS Code and the language server"
        }
      }
    }
  },
  "dependencies": {
    "vscode-languageclient": "^9.0.0"
  },
  "devDependencies": {
    "@types/vscode": "^1.80.0",
    "typescript": "^5.0.0"
  }
}
```

3. **Create `src/extension.ts`:**

```typescript
import * as path from 'path';
import { workspace, ExtensionContext } from 'vscode';
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
} from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: ExtensionContext) {
  // Get LSP server path from configuration
  const config = workspace.getConfiguration('glimmer-weave');
  const serverPath = config.get<string>('server.path') || 'glimmer-lsp';

  const serverOptions: ServerOptions = {
    command: serverPath,
    args: [],
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: 'file', language: 'glimmer-weave' }],
    synchronize: {
      fileEvents: workspace.createFileSystemWatcher('**/*.gw'),
    },
  };

  client = new LanguageClient(
    'glimmer-weave-lsp',
    'Glimmer-Weave Language Server',
    serverOptions,
    clientOptions
  );

  client.start();
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}
```

4. **Create `language-configuration.json`:**

```json
{
  "comments": {
    "lineComment": "#"
  },
  "brackets": [
    ["{", "}"],
    ["[", "]"],
    ["(", ")"]
  ],
  "autoClosingPairs": [
    { "open": "{", "close": "}" },
    { "open": "[", "close": "]" },
    { "open": "(", "close": ")" },
    { "open": "\"", "close": "\"", "notIn": ["string"] }
  ],
  "surroundingPairs": [
    ["{", "}"],
    ["[", "]"],
    ["(", ")"],
    ["\"", "\""]
  ]
}
```

5. **Build and run:**

```bash
npm install
npm run compile
code --extensionDevelopmentPath=/path/to/vscode-extension
```

## Testing the LSP Server

### Manual Testing

1. **Start the server:**

```bash
./target/release/glimmer-lsp
```

The server will wait for JSON-RPC messages on stdin.

2. **Send an initialize request:**

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {
    "processId": null,
    "rootUri": null,
    "capabilities": {}
  }
}
```

3. **Open a document:**

```json
{
  "jsonrpc": "2.0",
  "method": "textDocument/didOpen",
  "params": {
    "textDocument": {
      "uri": "file:///test.gw",
      "languageId": "glimmer-weave",
      "version": 1,
      "text": "bind x to 42\nx + 10"
    }
  }
}
```

The server will respond with diagnostics if there are any errors.

## Architecture

### Components

```
┌─────────────────────────────────────────────┐
│           VS Code / Editor                  │
│         (LSP Client)                        │
└─────────────────┬───────────────────────────┘
                  │ JSON-RPC over stdin/stdout
                  ↓
┌─────────────────────────────────────────────┐
│        Glimmer-Weave LSP Server             │
│         (src/lsp.rs)                        │
├─────────────────────────────────────────────┤
│  - Document synchronization                 │
│  - Request handlers (hover, completion)     │
│  - Diagnostics publisher                    │
└─────────────────┬───────────────────────────┘
                  │
      ┌───────────┴───────────┬────────────┐
      ↓                       ↓            ↓
┌──────────┐          ┌──────────────┐  ┌───────────────┐
│  Lexer   │    →     │    Parser    │  │   Semantic    │
│          │          │              │  │   Analyzer    │
└──────────┘          └──────────────┘  └───────┬───────┘
                                                 ↓
                                        ┌────────────────┐
                                        │ Type Inference │
                                        │    Engine      │
                                        └────────────────┘
```

### Request Flow

1. **Editor** sends `textDocument/didOpen` or `textDocument/didChange`
2. **LSP Server** updates its in-memory document store
3. **LSP Server** runs analysis pipeline:
   - Lexer tokenizes the document
   - Parser builds AST
   - Semantic analyzer checks for errors
   - Type inference validates types
4. **LSP Server** publishes diagnostics back to editor
5. **Editor** displays errors/warnings inline

## Development

### Adding New LSP Features

To add a new LSP feature (e.g., "Find References"):

1. **Add the handler to `src/lsp.rs`:**

```rust
async fn references(
    &self,
    params: ReferenceParams,
) -> JsonRpcResult<Option<Vec<Location>>> {
    // Implementation
    Ok(None)
}
```

2. **Enable the capability in `initialize()`:**

```rust
capabilities: ServerCapabilities {
    references_provider: Some(OneOf::Left(true)),
    // ... other capabilities
}
```

3. **Test with VS Code or your editor**

### Debugging

Set the trace level to `verbose` in your editor configuration to see all LSP messages:

```json
{
  "glimmer-weave.trace.server": "verbose"
}
```

View the output in your editor's LSP log panel.

## Known Limitations

1. **Position tracking**: Diagnostics currently use line 0, character 0 for all errors. Need to integrate source position tracking from parser.
2. **Hover information**: Currently returns placeholder text. Needs integration with type inference to show actual type information.
3. **Completion**: Only provides keywords, not context-aware suggestions (no variable/function suggestions yet).
4. **Performance**: Large files may be slow as we re-parse on every change (need incremental parsing).
5. **Cross-file navigation**: Go-to-definition only works within a single file currently.

## Future Improvements

- [ ] Incremental parsing for better performance
- [ ] Accurate source position tracking for diagnostics
- [ ] Symbol table for go-to-definition and find-references
- [ ] Context-aware completion (variables, functions in scope)
- [ ] Type information on hover
- [ ] Code actions (quick fixes)
- [ ] Formatting provider
- [ ] Workspace symbols search
- [ ] Multi-file analysis
- [ ] Project-wide refactoring

## Contributing

See [CLAUDE.md](CLAUDE.md) for development guidelines. When adding LSP features:

1. Follow Rust best practices
2. Run `cargo clippy --features lsp -- -D warnings` before committing
3. Ensure all tests pass: `cargo test --lib`
4. Document new features in this file
5. Update VS Code extension if needed

## References

- [LSP Specification](https://microsoft.github.io/language-server-protocol/)
- [tower-lsp Documentation](https://docs.rs/tower-lsp/)
- [VS Code Extension API](https://code.visualstudio.com/api)
