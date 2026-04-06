#!/usr/bin/env bash
#
# Sync tree-sitter grammar rev and query files from umple-lsp into umple.zed.
#
# Usage:
#   ./scripts/sync-grammar.sh --source /path/to/umple-lsp   # local sync
#   ./scripts/sync-grammar.sh --source /path/to/umple-lsp --check  # drift check (CI)
#   ./scripts/sync-grammar.sh --source /path/to/umple-lsp --rev abc123  # explicit rev
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
EXTENSION_TOML="$REPO_DIR/extension.toml"
DEST_DIR="$REPO_DIR/languages/umple"

# Query files to sync (add brackets.scm, indents.scm here when needed)
QUERY_FILES=(highlights.scm)

SOURCE=""
REV=""
CHECK=false

while [[ $# -gt 0 ]]; do
  case $1 in
    --source) SOURCE="$2"; shift 2 ;;
    --rev) REV="$2"; shift 2 ;;
    --check) CHECK=true; shift ;;
    *) echo "Unknown option: $1" >&2; exit 1 ;;
  esac
done

if [[ -z "$SOURCE" ]]; then
  echo "Error: --source /path/to/umple-lsp is required" >&2
  exit 1
fi

QUERIES_DIR="$SOURCE/packages/tree-sitter-umple/queries"

# Resolve rev from source repo if not explicitly provided
if [[ -z "$REV" ]]; then
  REV="$(cd "$SOURCE" && git rev-parse HEAD)"
fi

# Verify source files exist
for f in "${QUERY_FILES[@]}"; do
  if [[ ! -f "$QUERIES_DIR/$f" ]]; then
    echo "Error: source file not found: $QUERIES_DIR/$f" >&2
    exit 1
  fi
done

DRIFT=false

# --- Check / update extension.toml rev ---
CURRENT_REV="$(grep 'rev = ' "$EXTENSION_TOML" | sed 's/.*rev = "\(.*\)"/\1/')"
if [[ "$CURRENT_REV" != "$REV" ]]; then
  if $CHECK; then
    echo "DRIFT: extension.toml rev is $CURRENT_REV, expected $REV"
    DRIFT=true
  else
    TMP="$(mktemp)"
    sed "s/rev = \".*\"/rev = \"$REV\"/" "$EXTENSION_TOML" > "$TMP" && mv "$TMP" "$EXTENSION_TOML"
    echo "Updated extension.toml rev: $CURRENT_REV -> $REV"
  fi
else
  echo "extension.toml rev is current ($REV)"
fi

# --- Check / copy query files ---
mkdir -p "$DEST_DIR"

for f in "${QUERY_FILES[@]}"; do
  SRC="$QUERIES_DIR/$f"
  DST="$DEST_DIR/$f"

  # Build expected content with header
  HEADER="; Auto-synced from umple-lsp @ ${REV:0:12}
; Source: packages/tree-sitter-umple/queries/$f
; Do not edit manually — run scripts/sync-grammar.sh instead
"
  EXPECTED="${HEADER}$(cat "$SRC")"

  if [[ -f "$DST" ]] && [[ "$(cat "$DST")" == "$EXPECTED" ]]; then
    echo "$f is current"
  else
    if $CHECK; then
      echo "DRIFT: $f differs from source"
      DRIFT=true
    else
      echo "$EXPECTED" > "$DST"
      echo "Synced $f from umple-lsp @ ${REV:0:12}"
    fi
  fi
done

if $CHECK && $DRIFT; then
  echo ""
  echo "FAILED: drift detected. Run: ./scripts/sync-grammar.sh --source /path/to/umple-lsp"
  exit 1
elif $CHECK; then
  echo ""
  echo "OK: all synced files are current"
fi
