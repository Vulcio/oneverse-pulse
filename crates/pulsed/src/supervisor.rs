//! Only process lifecycle and the engine's public health contract live here.
use crate::engine_bundle;
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    process::Stdio,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{
    process::{Child, Command},
    sync::{watch, RwLock},
};

#[derive(Clone)]
pub struct Config {
    pub bundle: PathBuf,
    pub data: PathBuf,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Result<Option<Self>, Box<dyn std::error::Error>> {
        let Some(root) = std::env::var_os("PULSE_ENGINE_DIR") else {
            return Ok(None);
        };
        if root.is_empty() {
            return Err("PULSE_ENGINE_DIR must not be empty".into());
        }
        for name in ["ONEVERSE_API_URL", "PULSE_NODE_ID"] {
            if std::env::var(name).unwrap_or_default().trim().is_empty() {
                return Err(format!("{name} is required with PULSE_ENGINE_DIR").into());
            }
        }
        let config = Self {
            bundle: std::fs::canonicalize(root)?,
            data: std::env::var_os("PULSE_DATA_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|| "/var/lib/oneverse-pulse/engine".into()),
            port: std::env::var("PULSE_ROOM_PORT")
                .unwrap_or_else(|_| "8788".into())
                .parse()?,
        };
        if config.port == 0 || config.data.as_os_str().is_empty() {
            return Err("engine needs a nonzero port and a data directory".into());
        }
        Ok(Some(config))
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct EngineHealth {
    pub pairing: Option<Pairing>,
    pub name: Option<String>,
    pub set: Option<String>,
    pub cloud: String,
    pub session: Option<String>,
    pub epoch: Option<u64>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Pairing {
    pub code: String,
    #[serde(rename = "expiresAt")]
    pub expires_at: String,
}

#[derive(Clone, Serialize)]
pub struct Status {
    pub state: &'static str,
    pub pid: Option<u32>,
    pub restarts: u64,
    pub retry_in_seconds: u64,
    pub last_exit: Option<String>,
    pub error: Option<String>,
    pub health: Option<EngineHealth>,
}

impl Default for Status {
    fn default() -> Self {
        Self {
            state: "disabled",
            pid: None,
            restarts: 0,
            retry_in_seconds: 0,
            last_exit: None,
            error: None,
            health: None,
        }
    }
}
pub type Shared = Arc<RwLock<Status>>;

fn backoff(failures: u32) -> Duration {
    Duration::from_secs(1u64 << failures.min(5)).min(Duration::from_secs(30))
}

async fn stop(child: &mut Child) {
    if let Some(pid) = child.id() {
        // This is our unreaped child; its PID cannot be reused before wait().
        unsafe {
            libc::kill(pid as libc::pid_t, libc::SIGTERM);
        }
        if tokio::time::timeout(Duration::from_secs(5), child.wait())
            .await
            .is_err()
        {
            let _ = child.kill().await;
            let _ = child.wait().await;
        }
    }
}

async fn health(client: &reqwest::Client, port: u16) -> Result<EngineHealth, String> {
    let mut response = client
        .get(format!("http://127.0.0.1:{port}/health"))
        .send()
        .await
        .map_err(|_| "engine health unavailable")?
        .error_for_status()
        .map_err(|_| "engine health returned an error")?;
    let mut body = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| "engine health read failed")?
    {
        if body.len() + chunk.len() > 65_536 {
            return Err("engine health response too large".into());
        }
        body.extend_from_slice(&chunk);
    }
    serde_json::from_slice(&body).map_err(|_| "invalid engine health response".into())
}

pub async fn run(
    config: Config,
    expected: engine_bundle::Manifest,
    status: Shared,
    mut shutdown: watch::Receiver<bool>,
) {
    let client = reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(1))
        .build()
        .expect("local HTTP client");
    let mut failures = 0;
    let mut attempts: u64 = 0;
    while !*shutdown.borrow() {
        {
            let mut s = status.write().await;
            s.state = "starting";
            s.health = None;
            s.error = None;
            s.retry_in_seconds = 0;
            s.restarts = attempts.saturating_sub(1);
        }
        let verified = engine_bundle::verified(config.bundle.clone()).await;
        if *shutdown.borrow() {
            break;
        }
        let started = Instant::now();
        let spawn = verified.and_then(|manifest| {
            if manifest != expected {
                return Err("engine bundle changed; restart pulsed to select a new bundle".into());
            }
            // Refuse to supervise somebody else's room server on this port.
            let address = std::net::SocketAddr::from((std::net::Ipv4Addr::LOCALHOST, config.port));
            if std::net::TcpStream::connect_timeout(&address, Duration::from_millis(100)).is_ok() {
                return Err("engine room port already in use".into());
            }
            Command::new(config.bundle.join("runtime/bin/node"))
                .arg(config.bundle.join("engine/dist/main.js"))
                .current_dir(config.bundle.join("engine"))
                .env("PULSE_DATA_DIR", &config.data)
                .env("PULSE_ROOM_PORT", config.port.to_string())
                .env("PULSE_CONSOLE_DIR", config.bundle.join("console"))
                .stdin(Stdio::null())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .kill_on_drop(true)
                .spawn()
                .map_err(|e| format!("engine spawn failed: {e}"))
        });
        attempts += 1;
        match spawn {
            Ok(mut child) => {
                {
                    let mut s = status.write().await;
                    s.pid = child.id();
                    s.restarts = attempts - 1;
                }
                let mut tick = tokio::time::interval(Duration::from_secs(1));
                tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
                loop {
                    tokio::select! {
                        _ = shutdown.changed() => {
                            status.write().await.state = "stopping";
                            stop(&mut child).await;
                            let mut s = status.write().await; s.state = "stopped"; s.pid = None; s.health = None;
                            return;
                        }
                        exit = child.wait() => {
                            let mut s = status.write().await;
                            s.last_exit = Some(match exit { Ok(code) => code.to_string(), Err(e) => e.to_string() });
                            break;
                        }
                        _ = tick.tick() => {
                            let result = health(&client, config.port).await;
                            let mut s = status.write().await;
                            match result {
                                Ok(h) => { s.state = "running"; s.health = Some(h); s.error = None; }
                                Err(e) => { s.state = "unhealthy"; s.health = None; s.error = Some(e); }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                status.write().await.error = Some(e);
            }
        }
        if started.elapsed() >= Duration::from_secs(60) {
            failures = 0;
        }
        let delay = backoff(failures);
        failures = failures.saturating_add(1);
        {
            let mut s = status.write().await;
            s.state = "backoff";
            s.pid = None;
            s.health = None;
            s.retry_in_seconds = delay.as_secs();
            eprintln!("Pulse engine stopped; retry in {}s", delay.as_secs());
        }
        tokio::select! { _ = shutdown.changed() => break, _ = tokio::time::sleep(delay) => {} }
    }
    let mut s = status.write().await;
    s.state = "stopped";
    s.pid = None;
    s.health = None;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn restart_delay_is_bounded() {
        assert_eq!(backoff(0).as_secs(), 1);
        assert_eq!(backoff(2).as_secs(), 4);
        assert_eq!(backoff(20).as_secs(), 30);
    }
}
