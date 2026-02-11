# Umple for Zed

Umple language support for the [Zed](https://zed.dev) editor, providing syntax highlighting, diagnostics, code completion, and go-to-definition for `.ump` files.

## Installation

### From Extensions Marketplace

Search for "Umple" in Zed's extension browser (`Extensions` panel).

The extension automatically downloads the LSP server and Umple compiler — no manual setup required.

### Prerequisites

- **Node.js** 18+ (for running the LSP server)
- **Java** 11+ (optional — needed for diagnostics from the Umple compiler)

## Features

- **Syntax highlighting** via tree-sitter grammar
- **Diagnostics** from UmpleSync compiler
- **Code completion** with context-aware keyword suggestions
- **Go-to-definition** for classes, interfaces, traits, enums, attributes, methods, state machines, and `use` statements
- **Outline view** showing classes, methods, state machines, and more
- **Auto-indentation** for blocks

## How It Works

The extension automatically installs [`umple-lsp-server`](https://www.npmjs.com/package/umple-lsp-server) from npm and downloads `umplesync.jar` for compiler diagnostics. The server is launched via Node.js in `--stdio` mode.

## Configuration (optional)

For development, you can override the auto-downloaded server with a local build. Add to your Zed `settings.json` (`Cmd+,`):

```json
{
  "lsp": {
    "umple-lsp": {
      "settings": {
        "serverPath": "/path/to/umple-lsp"
      }
    }
  }
}
```

This points to a locally cloned and built [umple-lsp](https://github.com/umple/umple-lsp) repository.

## License

MIT
