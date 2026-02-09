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

        let server_path = Self::server_path_from_settings(&lsp_settings)?;
        let server_js = format!("{server_path}/packages/server/out/server.js");
        let jar_path = format!("{server_path}/packages/server/umplesync.jar");
        let node = Self::resolve_node(worktree)?;

        Ok(zed::Command::new(&node)
            .arg(&server_js)
            .arg("--stdio")
            .env("UMPLESYNC_JAR_PATH", &jar_path))
    }
}

impl UmpleExtension {
    fn server_path_from_settings(lsp_settings: &LspSettings) -> Result<String> {
        // Check settings.serverPath
        if let Some(ref settings) = lsp_settings.settings {
            if let Some(path) = settings.get("serverPath").and_then(|v| v.as_str()) {
                if !path.is_empty() {
                    return Ok(path.to_string());
                }
            }
        }

        // Check binary.path as fallback
        if let Some(ref binary) = lsp_settings.binary {
            if let Some(ref path) = binary.path {
                if !path.is_empty() {
                    return Ok(path.clone());
                }
            }
        }

        Err("umple-lsp server path not configured. \
             Add to your Zed settings: \
             {\"lsp\": {\"umple-lsp\": {\"settings\": {\"serverPath\": \"/path/to/umple-lsp\"}}}}"
            .to_string())
    }

    fn resolve_node(worktree: &zed::Worktree) -> Result<String> {
        if let Some(node) = worktree.which("node") {
            return Ok(node);
        }
        zed::node_binary_path().map_err(|e| format!("could not find node binary: {e}"))
    }
}

zed::register_extension!(UmpleExtension);
