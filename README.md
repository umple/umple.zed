# Umple for Zed

Umple language support for the [Zed](https://zed.dev) editor, providing syntax highlighting, diagnostics, code completion, and go-to-definition for `.ump` files.

## Installation

The extension is not yet available on the Zed marketplace. For now, install it manually as a dev extension:

1. Clone this repo:
   ```bash
   git clone https://github.com/umple/umple.zed.git
   ```

2. In Zed, open the command palette (`Cmd+Shift+P`) and run **zed: install dev extension**

3. Select the `umple.zed` directory

The extension automatically downloads the LSP server and Umple compiler — no manual setup required beyond the clone.

### Prerequisites

- **Rust** (via [rustup](https://rustup.rs/), not Homebrew — needed to compile the dev extension)
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

## Updating

Since this is installed as a dev extension, pull the latest changes and Zed will pick them up:

```bash
cd umple.zed
git pull
```

Then restart Zed or reload the extension.

## Troubleshooting

### Extension fails to compile ("failed to compile Rust extension")

Zed compiles extensions to WebAssembly (`wasm32-wasip2`), which requires the Rust toolchain from [rustup](https://rustup.rs/). Homebrew's `rust` package only includes the native target and can't cross-compile to WASM.

```bash
# Remove Homebrew rust if installed
brew uninstall rust

# Install via [rustup](https://rustup.rs/)
```

### LSP server not starting

Check **View > Toggle Language Server Logs** in Zed for errors. Common issues:
- Node.js not found: Install Node.js 18+ and ensure it's on your PATH
- npm install failed: Check internet connection, restart Zed to retry

### No diagnostics

Diagnostics require Java 11+. Check the LSP logs (**View > Toggle Language Server Logs**) for errors related to `umplesync`.

## License

MIT
