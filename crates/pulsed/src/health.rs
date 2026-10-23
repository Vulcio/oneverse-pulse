use crate::{engine_bundle::Manifest, supervisor};
use axum::{extract::State, http::StatusCode, Json};
use serde_json::json;
use std::{ffi::CString, path::PathBuf, sync::Arc, time::Instant};

pub struct Health {
    pub started: Instant,
    pub engine: supervisor::Shared,
    pub manifest: Option<Manifest>,
    pub data: PathBuf,
}
pub type Shared = Arc<Health>;

impl Health {
    pub fn standalone() -> Shared {
        Arc::new(Self {
            started: Instant::now(),
            engine: Arc::default(),
            manifest: None,
            data: ".".into(),
        })
    }
}

fn disk_available(path: &std::path::Path) -> Option<u64> {
    use std::os::unix::ffi::OsStrExt;
    let path = CString::new(path.as_os_str().as_bytes()).ok()?;
    let mut stats = std::mem::MaybeUninit::<libc::statvfs>::uninit();
    // statvfs initializes the structure on success; the C string lives through the call.
    if unsafe { libc::statvfs(path.as_ptr(), stats.as_mut_ptr()) } != 0 {
        return None;
    }
    let stats = unsafe { stats.assume_init() };
    Some(stats.f_bavail.saturating_mul(stats.f_frsize))
}

pub async fn response(State(state): State<Shared>) -> (StatusCode, Json<serde_json::Value>) {
    let engine = state.engine.read().await.clone();
    let ready = state.manifest.is_none() || engine.state == "running";
    let temperature = tokio::fs::read_to_string("/sys/class/thermal/thermal_zone0/temp")
        .await
        .ok()
        .and_then(|v| v.trim().parse::<f64>().ok())
        .map(|v| v / 1000.0);
    let bundle = state
        .manifest
        .as_ref()
        .map(|m| json!({ "source_revision": m.source_revision, "node_version": m.node_version }));
    (
        if ready {
            StatusCode::OK
        } else {
            StatusCode::SERVICE_UNAVAILABLE
        },
        Json(json!({
            "service": "oneverse-pulse", "version": env!("CARGO_PKG_VERSION"),
            "status": if ready { "ok" } else { "degraded" },
            "uptime_seconds": state.started.elapsed().as_secs(),
            "disk_available_bytes": disk_available(&state.data),
            "temperature_celsius": temperature, "bundle": bundle, "engine": engine
        })),
    )
}
