use std::fs;
use zed_extension_api::{self as zed, settings::LspSettings, LanguageServerId, Result};

struct UmpleExtension;

impl zed::Extension for UmpleExtension {
    fn new() -> Self {
        UmpleExtension
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let lsp_settings = LspSettings::for_worktree(language_server_id.as_ref(), worktree)
            .map_err(|e| format!("failed to get LSP settings: {e}"))?;

        // Determine the server path from settings or binary config
        let server_path = Self::server_path_from_settings(&lsp_settings)?;

        // Resolve the server entry point
        let server_js = format!("{server_path}/packages/server/out/server.js");

        // Resolve the JAR path
        let jar_path = format!("{server_path}/packages/server/umplesync.jar");

        // Find node binary: prefer user-configured path, then Zed's bundled node
        let node = Self::resolve_node(&lsp_settings, worktree)?;

        Ok(zed::Command::new(&node)
            .arg(&server_js)
            .arg("--stdio")
            .env("UMPLESYNC_JAR_PATH", &jar_path))
    }
}

impl UmpleExtension {
    /// Extract the server path from LSP settings.
    ///
    /// Checks (in order):
    /// 1. `lsp.umple-lsp.settings.serverPath`
    /// 2. `lsp.umple-lsp.binary.path` (treated as the repo root)
    fn server_path_from_settings(lsp_settings: &LspSettings) -> Result<String> {
        // Check settings.serverPath first
        if let Some(ref settings) = lsp_settings.settings {
            if let Some(path) = settings.get("serverPath").and_then(|v| v.as_str()) {
                let path = path.to_string();
                // Validate the path exists
                if fs::metadata(&path).is_ok() {
                    return Ok(path);
                }
                return Err(format!(
                    "serverPath '{path}' does not exist. \
                     Please set a valid path in Zed settings: \
                     lsp.umple-lsp.settings.serverPath"
                ));
            }
        }

        // Check binary.path as fallback
        if let Some(ref binary) = lsp_settings.binary {
            if let Some(ref path) = binary.path {
                if fs::metadata(path).is_ok() {
                    return Ok(path.clone());
                }
            }
        }

        Err(
            "umple-lsp server path not configured. \
             Add to your Zed settings: \
             {\"lsp\": {\"umple-lsp\": {\"settings\": {\"serverPath\": \"/path/to/umple-lsp\"}}}}"
                .to_string(),
        )
    }

    /// Resolve the node binary path.
    ///
    /// Checks (in order):
    /// 1. Worktree PATH lookup
    /// 2. Zed's bundled node
    fn resolve_node(_lsp_settings: &LspSettings, worktree: &zed::Worktree) -> Result<String> {
        // Try worktree PATH
        if let Some(node) = worktree.which("node") {
            return Ok(node);
        }

        // Fall back to Zed's bundled node
        zed::node_binary_path().map_err(|e| format!("could not find node binary: {e}"))
    }
}

zed::register_extension!(UmpleExtension);
