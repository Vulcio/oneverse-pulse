mod config;
mod dev;
mod engine_bundle;
mod health;
mod offline;
mod supervisor;

use axum::{routing::get, Router};
use config::Command;

fn routes(dev: bool, health: health::Shared) -> Router {
    let app = Router::new()
        .route("/health", get(health::response))
        .with_state(health);
    if dev {
        app.merge(dev::routes())
    } else {
        app
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dev = match config::command(std::env::args().skip(1))? {
        Command::Run { dev } => dev,
        Command::Help => {
            println!("Usage: pulsed [--dev]\n\n  --dev      Enable the mock set, navigation API and /screen\n  --help     Show this help\n  --version  Show the daemon version\n\nPULSE_BUNDLE_DIR serves a verified bundle independently of --dev.");
            return Ok(());
        }
        Command::Version => {
            println!("pulsed {}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
    };
    let mut engine = supervisor::Config::from_env()?;
    let mut state = health::Health::standalone();
    if let Some(config) = &mut engine {
        use std::os::unix::fs::PermissionsExt;
        tokio::fs::create_dir_all(&config.data).await?;
        tokio::fs::set_permissions(&config.data, std::fs::Permissions::from_mode(0o700)).await?;
        config.data = tokio::fs::canonicalize(&config.data).await?;
        if config.data.starts_with(&config.bundle) {
            return Err("PULSE_DATA_DIR must be outside the engine bundle".into());
        }
        let manifest = engine_bundle::verified(config.bundle.clone()).await?;
        state = std::sync::Arc::new(health::Health {
            started: std::time::Instant::now(),
            engine: std::sync::Arc::default(),
            manifest: Some(manifest),
            data: config.data.clone(),
        });
    }
    let mut app = routes(dev, state.clone());
    if let Some(dir) = config::bundle_dir() {
        // Never expose a directory that has not passed manifest verification.
        let bundle = offline::Bundle::open(&dir).await?;
        app = app.merge(offline::routes(bundle));
        println!("Verified local screen bundle: {}", dir.to_string_lossy());
    }
    let port: u16 = std::env::var("PULSE_DAEMON_PORT")
        .unwrap_or_else(|_| "8787".into())
        .parse()?;
    if port == 0 {
        return Err("PULSE_DAEMON_PORT must be nonzero".into());
    }
    let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, port)).await?;
    println!("Oneverse Pulse: http://127.0.0.1:{port}/health");
    if dev {
        println!("Development mocks enabled: http://127.0.0.1:{port}/screen");
    }
    let (shutdown, receiver) = tokio::sync::watch::channel(false);
    let task = engine.map(|config| {
        tokio::spawn(supervisor::run(
            config,
            state
                .manifest
                .as_ref()
                .expect("configured engine manifest")
                .clone(),
            state.engine.clone(),
            receiver,
        ))
    });
    let shutdown_on_signal = shutdown.clone();
    let result = axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            use tokio::signal::unix::{signal, SignalKind};
            let mut term = signal(SignalKind::terminate()).expect("SIGTERM handler");
            tokio::select! { _ = tokio::signal::ctrl_c() => {}, _ = term.recv() => {} }
            let _ = shutdown_on_signal.send(true);
        })
        .await;
    let _ = shutdown.send(true);
    if let Some(task) = task {
        task.await?;
    }
    result?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{to_bytes, Body},
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    async fn request(app: Router, method: &str, path: &str) -> axum::response::Response {
        app.oneshot(
            Request::builder()
                .method(method)
                .uri(path)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap()
    }

    async fn json(response: axum::response::Response) -> serde_json::Value {
        let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
        serde_json::from_slice(&body).unwrap()
    }

    #[tokio::test]
    async fn mocks_are_absent_by_default() {
        let app = routes(false, health::Health::standalone());
        assert_eq!(
            request(app.clone(), "GET", "/health").await.status(),
            StatusCode::OK
        );
        for (method, path) in [
            ("GET", "/screen"),
            ("GET", "/api/state"),
            ("POST", "/api/next"),
            ("POST", "/api/previous"),
        ] {
            assert_eq!(
                request(app.clone(), method, path).await.status(),
                StatusCode::NOT_FOUND
            );
        }
    }

    #[tokio::test]
    async fn dev_keeps_the_prototype_navigation_contract() {
        let app = routes(true, health::Health::standalone());
        assert_eq!(
            request(app.clone(), "GET", "/screen").await.status(),
            StatusCode::OK
        );
        assert_eq!(
            json(request(app.clone(), "GET", "/api/state").await).await["index"],
            0
        );
        assert_eq!(
            json(request(app.clone(), "POST", "/api/next").await).await,
            1
        );
        assert_eq!(
            json(request(app.clone(), "GET", "/api/state").await).await["index"],
            1
        );
        for _ in 0..4 {
            request(app.clone(), "POST", "/api/next").await;
        }
        assert_eq!(
            json(request(app.clone(), "GET", "/api/state").await).await["index"],
            2
        );
        for _ in 0..4 {
            request(app.clone(), "POST", "/api/previous").await;
        }
        assert_eq!(
            json(request(app, "GET", "/api/state").await).await["index"],
            0
        );
    }

    #[tokio::test]
    async fn verified_bundles_work_without_mocks_and_reject_modified_assets() {
        use sha2::{Digest, Sha256};
        use std::time::{SystemTime, UNIX_EPOCH};

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("pulse-bundle-{}-{unique}", std::process::id()));
        tokio::fs::create_dir_all(root.join("assets"))
            .await
            .unwrap();
        let index = "<html>Verified test bundle</html>";
        let asset = "console.log('verified');";
        tokio::fs::write(root.join("index.html"), index)
            .await
            .unwrap();
        tokio::fs::write(root.join("assets/app.js"), asset)
            .await
            .unwrap();
        let manifest = serde_json::json!({
            "format": 1,
            "source_revision": "test-fixture",
            "files": {
                "index.html": hex::encode(Sha256::digest(index.as_bytes())),
                "assets/app.js": hex::encode(Sha256::digest(asset.as_bytes()))
            }
        });
        tokio::fs::write(root.join("manifest.json"), manifest.to_string())
            .await
            .unwrap();

        let app = routes(false, health::Health::standalone())
            .merge(offline::routes(offline::Bundle::open(&root).await.unwrap()));
        let response = request(app.clone(), "GET", "/offline/screen").await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            to_bytes(response.into_body(), 1024).await.unwrap().as_ref(),
            index.as_bytes()
        );
        assert_eq!(
            request(app.clone(), "GET", "/assets/app.js").await.status(),
            StatusCode::OK
        );
        assert_eq!(
            request(app.clone(), "GET", "/assets/unknown.js")
                .await
                .status(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            request(app.clone(), "GET", "/api/state").await.status(),
            StatusCode::NOT_FOUND
        );

        tokio::fs::write(root.join("assets/app.js"), "tampered")
            .await
            .unwrap();
        assert_eq!(
            request(app, "GET", "/assets/app.js").await.status(),
            StatusCode::SERVICE_UNAVAILABLE
        );
        assert!(offline::Bundle::open(&root).await.is_err());
        tokio::fs::remove_dir_all(root).await.unwrap();
    }
}
