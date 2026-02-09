# Umple for Zed

Umple language support for the [Zed](https://zed.dev) editor, providing syntax highlighting, diagnostics, code completion, and go-to-definition for `.ump` files.

## Prerequisites

- **Node.js** 18+ (for running the LSP server)
- **Java** 11+ (for UmpleSync diagnostics)
- **umple-lsp** cloned and built locally:

```bash
git clone https://github.com/DraftTin/umple-lsp.git
cd umple-lsp
npm install
npm run compile
npm run download-jar
```

## Installation

### Dev Extension (local development)

1. Clone this repo
2. In Zed: `Extensions` > `Install Dev Extension` > select the `umple.zed` directory

### From Extensions Marketplace

Search for "Umple" in Zed's extension browser (once published).

## Configuration

Add to your Zed `settings.json` (`Cmd+,`):

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

Replace `/path/to/umple-lsp` with the absolute path to your cloned and built `umple-lsp` repository.

## Features

- **Syntax highlighting** via tree-sitter grammar
- **Diagnostics** from UmpleSync compiler
- **Code completion** with context-aware keyword suggestions
- **Go-to-definition** for classes, interfaces, traits, enums, attributes, methods, state machines, and `use` statements
- **Outline view** showing classes, methods, state machines, and more
- **Auto-indentation** for blocks

## How It Works

The extension launches the Umple LSP server (`packages/server/out/server.js`) via Node.js in `--stdio` mode. The server uses:

- **tree-sitter-umple** (WASM) for parsing and symbol indexing
- **umplesync.jar** for compiler diagnostics via socket connection
