use std::env;
use zed_extension_api::{
    self as zed,
    http_client::{HttpMethod, HttpRequest, RedirectPolicy},
    serde_json,
    settings::LspSettings,
    LanguageServerId, Result,
};

const PACKAGE_NAME: &str = "umple-lsp-server";
const SERVER_PATH: &str = "node_modules/umple-lsp-server/out/server.js";
const JAR_URL: &str = "https://try.umple.org/scripts/umplesync.jar";
const JAR_PATH: &str = "umplesync.jar";
const JAR_VERSION_URL: &str =
    "https://cruise.umple.org/umpleonline/scripts/versionRunning.txt";
const JAR_VERSION_CACHE: &str = "umplesync.jar.version";

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
            let node = Self::resolve_node()?;
            return Ok(zed::Command::new(&node).arg(&server_js).arg("--stdio"));
        }

        // Auto-download path
        let server_path = self.server_script_path(language_server_id)?;
        let node =
            zed::node_binary_path().map_err(|e| format!("could not find node binary: {e}"))?;

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

    fn server_script_path(&mut self, language_server_id: &LanguageServerId) -> Result<String> {
        let server_exists = self.server_exists();
        if self.did_find_server && server_exists {
            self.refresh_jar_if_stale();
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

        self.refresh_jar_if_stale();
        self.did_find_server = true;
        Ok(SERVER_PATH.to_string())
    }

    /// Read the cached jar version recorded next to the jar, if any.
    /// Returns `None` when the cache file is missing, unreadable, or empty.
    fn read_cached_jar_version() -> Option<String> {
        let raw = std::fs::read_to_string(JAR_VERSION_CACHE).ok()?;
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    }

    /// Fetch the current jar version string from the Umple version endpoint.
    /// Returns `None` on any network or parse failure — callers must treat
    /// this as "check skipped, keep existing jar" (fail-open).
    fn fetch_remote_jar_version() -> Option<String> {
        let req = HttpRequest::builder()
            .method(HttpMethod::Get)
            .url(JAR_VERSION_URL)
            .redirect_policy(RedirectPolicy::FollowAll)
            .build()
            .ok()?;
        let resp = req.fetch().ok()?;
        let body = String::from_utf8(resp.body).ok()?;
        let trimmed = body.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    }

    /// Attempt to download the jar, retrying up to 3 times. Returns `true`
    /// only when `zed::download_file` succeeds this call; does not report on
    /// whether a pre-existing jar is still in place.
    fn download_jar(&self) -> bool {
        for _ in 1..=3 {
            if zed::download_file(JAR_URL, JAR_PATH, zed::DownloadedFileType::Uncompressed)
                .is_ok()
            {
                return true;
            }
        }
        false
    }

    /// Ensure the local jar is present and reasonably fresh.
    ///
    /// - If the jar is missing, download it unconditionally (same as before).
    /// - If the jar is present, fetch the remote version string; if it differs
    ///   from the cached value, re-download. Write the cache only after a
    ///   successful download so a failed refresh leaves the previous jar (and
    ///   its recorded version) intact.
    /// - All network failures are fail-open: keep the existing jar rather
    ///   than breaking diagnostics.
    fn refresh_jar_if_stale(&self) {
        let jar_missing = std::fs::metadata(JAR_PATH).map_or(true, |s| !s.is_file());

        if jar_missing {
            if self.download_jar() {
                if let Some(remote) = Self::fetch_remote_jar_version() {
                    let _ = std::fs::write(JAR_VERSION_CACHE, remote);
                }
            }
            return;
        }

        let Some(remote) = Self::fetch_remote_jar_version() else {
            return;
        };
        if Self::read_cached_jar_version().as_deref() == Some(remote.as_str()) {
            return;
        }

        if self.download_jar() {
            let _ = std::fs::write(JAR_VERSION_CACHE, remote);
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

    fn resolve_node() -> Result<String> {
        zed::node_binary_path().map_err(|e| format!("could not find node binary: {e}"))
    }
}

zed::register_extension!(UmpleExtension);
