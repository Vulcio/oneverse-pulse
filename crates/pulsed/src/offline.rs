//! Verified, read-only local web assets. This is transport, not the screen renderer.
//! A bundle is provisioned externally; the real Songverse screen frontend is not built here.
use axum::{
    extract::{Path as UrlPath, State},
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    path::{Component, Path, PathBuf},
    sync::Arc,
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    format: u32,
    source_revision: String,
    files: HashMap<String, String>,
}

pub struct Bundle {
    root: PathBuf,
    paths: HashMap<String, String>,
}
type SharedBundle = Arc<Bundle>;

fn safe_relative(path: &str) -> bool {
    !path.is_empty()
        && !path.contains('\\')
        && Path::new(path)
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
}

impl Bundle {
    pub async fn open(root: impl AsRef<Path>) -> Result<SharedBundle, Box<dyn std::error::Error>> {
        let root = tokio::fs::canonicalize(root).await?;
        let raw = tokio::fs::read(root.join("manifest.json")).await?;
        let manifest: Manifest = serde_json::from_slice(&raw)?;
        if manifest.format != 1 || manifest.source_revision.trim().is_empty() {
            return Err("unsupported or incomplete bundle manifest".into());
        }
        if !manifest.files.contains_key("index.html") {
            return Err("bundle needs index.html".into());
        }
        if manifest.files.len() > 20_000 {
            return Err("bundle has too many files".into());
        }
        for (path, digest) in &manifest.files {
            if !safe_relative(path)
                || digest.len() != 64
                || !digest.bytes().all(|b| b.is_ascii_hexdigit())
            {
                return Err(format!("invalid manifest file entry: {path}").into());
            }
            let absolute = tokio::fs::canonicalize(root.join(path)).await?;
            if !absolute.starts_with(&root) || !absolute.is_file() {
                return Err(format!("path escapes bundle: {path}").into());
            }
            let bytes = tokio::fs::read(&absolute).await?;
            if hex::encode(Sha256::digest(&bytes)) != digest.to_ascii_lowercase() {
                return Err(format!("checksum mismatch: {path}").into());
            }
        }
        Ok(Arc::new(Self {
            root,
            paths: manifest.files,
        }))
    }

    async fn response(&self, path: &str) -> Response {
        if !safe_relative(path) || !self.paths.contains_key(path) {
            return StatusCode::NOT_FOUND.into_response();
        }
        let Ok(absolute) = tokio::fs::canonicalize(self.root.join(path)).await else {
            return StatusCode::NOT_FOUND.into_response();
        };
        if !absolute.starts_with(&self.root) || !absolute.is_file() {
            return StatusCode::FORBIDDEN.into_response();
        }
        let Ok(bytes) = tokio::fs::read(&absolute).await else {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        };
        // File bytes are checked again to avoid serving externally modified resources.
        if hex::encode(Sha256::digest(&bytes)) != self.paths[path].to_ascii_lowercase() {
            return StatusCode::SERVICE_UNAVAILABLE.into_response();
        }
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        let mut response = bytes.into_response();
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_str(mime.as_ref())
                .unwrap_or(HeaderValue::from_static("application/octet-stream")),
        );
        response
            .headers_mut()
            .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
        response
    }
}

async fn index(State(bundle): State<SharedBundle>) -> Response {
    bundle.response("index.html").await
}
async fn asset(State(bundle): State<SharedBundle>, UrlPath(path): UrlPath<String>) -> Response {
    bundle.response(&format!("assets/{path}")).await
}
pub fn routes(bundle: SharedBundle) -> Router {
    Router::new()
        .route("/offline/screen", get(index))
        .route("/assets/*path", get(asset))
        .with_state(bundle)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_unsafe_paths() {
        assert!(safe_relative("assets/app.js"));
        assert!(!safe_relative("../secrets"));
        assert!(!safe_relative("/etc/passwd"));
        assert!(!safe_relative("assets/../../token"));
        assert!(!safe_relative("assets\\bad.js"));
        assert!(!safe_relative(""));
    }
}
