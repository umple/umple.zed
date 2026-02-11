use std::env;
use zed_extension_api::{self as zed, serde_json, settings::LspSettings, LanguageServerId, Result};

const PACKAGE_NAME: &str = "umple-lsp-server";
const SERVER_PATH: &str = "node_modules/umple-lsp-server/out/server.js";
const JAR_URL: &str = "https://try.umple.org/scripts/umplesync.jar";
const JAR_PATH: &str = "umplesync.jar";

struct UmpleExtension {
    did_find_server: bool,
}

impl zed::Extension for UmpleExtension {
    fn new() -> Self {
        UmpleExtension {
            did_find_server: false,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let lsp_settings = LspSettings::for_worktree(language_server_id.as_ref(), worktree)
            .map_err(|e| format!("failed to get LSP settings: {e}"))?;

        // Allow developers to override with a manual serverPath
        if let Some(path) = Self::server_path_from_settings(&lsp_settings) {
            let server_js = format!("{path}/packages/server/out/server.js");
            let node = Self::resolve_node(worktree)?;
            return Ok(zed::Command::new(&node)
                .arg(&server_js)
                .arg("--stdio"));
        }

        // Auto-download path
        let server_path = self.server_script_path(language_server_id)?;
        let node = zed::node_binary_path()
            .map_err(|e| format!("could not find node binary: {e}"))?;

        Ok(zed::Command::new(&node)
            .arg(
                &env::current_dir()
                    .unwrap()
                    .join(&server_path)
                    .to_string_lossy()
                    .to_string(),
            )
            .arg("--stdio"))
    }

    fn language_server_initialization_options(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<serde_json::Value>> {
        let lsp_settings = LspSettings::for_worktree(language_server_id.as_ref(), worktree)
            .map_err(|e| format!("failed to get LSP settings: {e}"))?;

        // If developer override, jar is inside the repo
        if let Some(path) = Self::server_path_from_settings(&lsp_settings) {
            return Ok(Some(serde_json::json!({
                "umpleSyncJarPath": format!("{path}/packages/server/umplesync.jar")
            })));
        }

        // Auto-download path: jar is in extension working directory
        let jar_path = env::current_dir()
            .unwrap()
            .join(JAR_PATH)
            .to_string_lossy()
            .to_string();

        Ok(Some(serde_json::json!({
            "umpleSyncJarPath": jar_path
        })))
    }
}

impl UmpleExtension {
    fn server_exists(&self) -> bool {
        std::fs::metadata(SERVER_PATH).map_or(false, |stat| stat.is_file())
    }

    fn server_script_path(
        &mut self,
        language_server_id: &LanguageServerId,
    ) -> Result<String> {
        let server_exists = self.server_exists();

        if self.did_find_server && server_exists {
            self.download_jar_if_needed();
            return Ok(SERVER_PATH.to_string());
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );
        let version = zed::npm_package_latest_version(PACKAGE_NAME)?;

        if !server_exists
            || zed::npm_package_installed_version(PACKAGE_NAME)?.as_ref() != Some(&version)
        {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );
            let result = zed::npm_install_package(PACKAGE_NAME, &version);
            match result {
                Ok(()) => {
                    if !self.server_exists() {
                        Err(format!(
                            "installed package '{PACKAGE_NAME}' did not contain expected path '{SERVER_PATH}'",
                        ))?;
                    }
                }
                Err(error) => {
                    if !self.server_exists() {
                        Err(error)?;
                    }
                }
            }
        }

        self.download_jar_if_needed();
        self.did_find_server = true;
        Ok(SERVER_PATH.to_string())
    }

    fn download_jar_if_needed(&self) {
        if std::fs::metadata(JAR_PATH).map_or(true, |stat| !stat.is_file()) {
            // Best-effort: diagnostics will be disabled if this fails
            let _ = zed::download_file(JAR_URL, JAR_PATH, zed::DownloadedFileType::Uncompressed);
        }
    }

    fn server_path_from_settings(lsp_settings: &LspSettings) -> Option<String> {
        if let Some(ref settings) = lsp_settings.settings {
            if let Some(path) = settings.get("serverPath").and_then(|v| v.as_str()) {
                if !path.is_empty() {
                    return Some(path.to_string());
                }
            }
        }
        if let Some(ref binary) = lsp_settings.binary {
            if let Some(ref path) = binary.path {
                if !path.is_empty() {
                    return Some(path.clone());
                }
            }
        }
        None
    }

    fn resolve_node(worktree: &zed::Worktree) -> Result<String> {
        if let Some(node) = worktree.which("node") {
            return Ok(node);
        }
        zed::node_binary_path().map_err(|e| format!("could not find node binary: {e}"))
    }
}

zed::register_extension!(UmpleExtension);
