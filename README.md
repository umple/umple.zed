# Umple for Zed

Umple language support for the [Zed](https://zed.dev) editor, providing syntax highlighting, diagnostics, code completion, and go-to-definition for `.ump` files.

## Installation

The extension is available on the Zed marketplace. Install it from Zed:

1. Open the command palette (`Cmd+Shift+P`) and run **zed: extensions**
2. Search for **Umple** and click Install

The extension automatically downloads the LSP server and Umple compiler — no manual setup required.

### Dev install (from source)

To work on the extension itself, install it as a dev extension:

1. Clone this repo:
   ```bash
   git clone https://github.com/umple/umple.zed.git
   ```

2. In Zed, open the command palette (`Cmd+Shift+P`) and run **zed: install dev extension**

3. Select the `umple.zed` directory

### Prerequisites

- **Rust** (via [rustup](https://rustup.rs/), not Homebrew rust — needed to compile the dev extension)
- **Node.js** 20+ (for running the LSP server; tested on 20 and 23)
- **Java** 11+ (optional — needed for diagnostics from the Umple compiler)

## Features

- **Syntax highlighting** via tree-sitter grammar
- **Diagnostics** from UmpleSync compiler
- **Code completion** with context-aware keyword suggestions
- **Go-to-definition** for classes, interfaces, traits, enums, attributes, methods, state machines, states, associations, mixsets, requirements, and `use` statements
- **Find references** across all reachable files with state-path disambiguation
- **Rename** safe symbol rename across all references
- **Hover** contextual information for symbols
- **Formatting** AST-driven indent correction, arrow spacing, blank-line normalization
- **Outline view** showing classes, methods, state machines, and more
- **Cross-file support** transitive `use` statement resolution and cross-file diagnostics
- **Auto-indentation** for blocks

## How It Works

The extension automatically installs [`umple-lsp-server`](https://www.npmjs.com/package/umple-lsp-server) from npm and downloads `umplesync.jar` for compiler diagnostics. The server is launched via Node.js in `--stdio` mode.

## Configuration (optional)

You can adjust certain settings using Settings ... / Open Settings (cmd ,) or by editing the `settings.json` file.

### Configuration settings for users

By default lines of Umple code that have errors or warnings are underlined; you can see the error or warning at the bottom of the screen if you click on the underlined text.

However, if you would like such messages from the Umple compiler to appear inline (on the line where each problem occurs), then you can change the Languages & Tools / Diagnostics / Enabled setting to be true. You can also do this by adding the following to the settings.json file.

```
{
  "diagnostics": {
    "inline": {
      "enabled": true
    }
  }
}
```

### Configuration settings for developers working on the Umple LSP server
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

**Marketplace install:** Zed updates installed extensions automatically. New versions reach you via the Zed Extensions marketplace.

**Dev install (from source):** pull the latest changes and Zed will pick them up:

```bash
cd umple.zed
git pull
```

Then restart Zed or reload the extension.

The LSP server itself is downloaded from npm at every extension load (via `npm_package_latest_version("umple-lsp-server")`), so server-only changes reach you on the next Zed startup without any extension update needed.

## Grammar Sync

The tree-sitter grammar and query files are derived from [umple-lsp](https://github.com/umple/umple-lsp). The following files are auto-synced and should not be edited manually:

- `extension.toml` `[grammars.umple].rev` — pinned commit for `parser.c`
- `languages/umple/highlights.scm` — copied from `umple-lsp/packages/tree-sitter-umple/queries/`

### How sync usually happens

A GitHub Action in [`umple/umple-lsp`](https://github.com/umple/umple-lsp/blob/master/.github/workflows/sync-umple-zed.yml) auto-opens (or updates) a PR on this repo whenever grammar or query files change upstream. The PR force-pushes onto a stable branch `sync/umple-lsp-master`, bumps `extension.toml` patch version, and includes a body that classifies the change as `grammar | highlights | both` plus a "safe to merge?" hint. You just merge it.

### Manually syncing (rarely needed)

If the auto-PR workflow is broken or you want to sync ahead of an upstream push:

```bash
./scripts/sync-grammar.sh --source /path/to/umple-lsp
```

### Checking for drift (CI or local)

```bash
./scripts/sync-grammar.sh --source /path/to/umple-lsp --check
```

This exits with code 1 if any synced file is out of date. The `.github/workflows/check-sync.yml` workflow in this repo runs this on every push/PR to master to catch drift.

## Troubleshooting

### Extension fails to compile ("failed to compile Rust extension")

Zed compiles extensions to WebAssembly (`wasm32-wasip2`), which requires the Rust toolchain from [rustup](https://rustup.rs/). Homebrew's `rust` package only includes the native target and can't cross-compile to WASM.

```bash
# Remove Homebrew rust if installed
brew uninstall rust

brew install rustup
# Or install rustup directly via [rustup](https://rustup.rs/)
```

### LSP server not starting

Check **View > Toggle Language Server Logs** in Zed for errors. Common issues:

- **Node.js not found:** Install Node.js 18+ and ensure it's on your PATH
- **npm install failed:** Check internet connection, restart Zed to retry

### No diagnostics

Diagnostics require Java 11+. Check the LSP logs (**View > Toggle Language Server Logs**) for errors related to `umplesync`.

## License

MIT
