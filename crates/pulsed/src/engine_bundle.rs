//! Executable bundles are installed by a trusted administrator, never by the service user.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    io::Read,
    path::{Component, Path, PathBuf},
};

#[derive(Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub format: u32,
    pub source_revision: String,
    pub node_version: String,
    pub platform: String,
    pub architecture: String,
    pub files: BTreeMap<String, String>,
}

pub fn files(root: &Path, dir: &Path, result: &mut BTreeMap<String, String>) -> Result<(), String> {
    for entry in std::fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let metadata = std::fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
        if metadata.file_type().is_symlink() {
            return Err("engine bundle contains a symlink".into());
        }
        if metadata.is_dir() {
            files(root, &path, result)?;
        } else if metadata.is_file() {
            let relative = path
                .strip_prefix(root)
                .map_err(|e| e.to_string())?
                .to_str()
                .ok_or("non-UTF8 bundle path")?
                .to_owned();
            if relative == "manifest.json" {
                continue;
            }
            if result.len() >= 20_000 {
                return Err("engine bundle has too many files".into());
            }
            let mut file = std::fs::File::open(path).map_err(|e| e.to_string())?;
            let mut hash = Sha256::new();
            let mut buffer = [0; 65536];
            loop {
                let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
                if n == 0 {
                    break;
                }
                hash.update(&buffer[..n]);
            }
            result.insert(relative, hex::encode(hash.finalize()));
        } else {
            return Err("engine bundle contains a special file".into());
        }
    }
    Ok(())
}

pub fn verify(root: &Path) -> Result<Manifest, String> {
    let manifest: Manifest = serde_json::from_slice(
        &std::fs::read(root.join("manifest.json")).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    if manifest.format != 1
        || manifest.source_revision.len() != 40
        || !manifest
            .source_revision
            .bytes()
            .all(|b| b.is_ascii_hexdigit())
        || !manifest.node_version.starts_with('v')
        || manifest.platform != std::env::consts::OS
        || manifest.architecture != std::env::consts::ARCH
    {
        return Err("incompatible engine bundle manifest".into());
    }
    for required in [
        "runtime/bin/node",
        "engine/dist/main.js",
        "console/pulse.html",
    ] {
        if !manifest.files.contains_key(required) {
            return Err(format!("engine bundle missing {required}"));
        }
    }
    if manifest.files.keys().any(|name| {
        name.contains('\\')
            || !Path::new(name)
                .components()
                .all(|c| matches!(c, Component::Normal(_)))
    }) {
        return Err("invalid engine bundle path".into());
    }
    let mut actual = BTreeMap::new();
    files(root, root, &mut actual)?;
    if actual != manifest.files {
        return Err("engine bundle checksum or file list mismatch".into());
    }
    Ok(manifest)
}

pub async fn verified(root: PathBuf) -> Result<Manifest, String> {
    tokio::task::spawn_blocking(move || verify(&root))
        .await
        .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn executable_bundles_reject_tampering_extra_files_and_links() {
        use std::time::{SystemTime, UNIX_EPOCH};
        let root = std::env::temp_dir().join(format!(
            "pulse-engine-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        for name in [
            "runtime/bin/node",
            "engine/dist/main.js",
            "console/pulse.html",
        ] {
            let path = root.join(name);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, b"test fixture").unwrap();
        }
        let mut expected = BTreeMap::new();
        files(&root, &root, &mut expected).unwrap();
        let manifest = Manifest {
            format: 1,
            source_revision: "a".repeat(40),
            node_version: "v24.19.0".into(),
            platform: std::env::consts::OS.into(),
            architecture: std::env::consts::ARCH.into(),
            files: expected,
        };
        std::fs::write(
            root.join("manifest.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        assert!(verify(&root).is_ok());
        std::fs::write(root.join("engine/extra.js"), b"unlisted").unwrap();
        assert!(verify(&root).is_err());
        std::fs::remove_file(root.join("engine/extra.js")).unwrap();
        std::os::unix::fs::symlink("dist/main.js", root.join("engine/linked.js")).unwrap();
        assert!(verify(&root).is_err());
        std::fs::remove_file(root.join("engine/linked.js")).unwrap();
        std::fs::write(root.join("runtime/bin/node"), b"changed").unwrap();
        assert!(verify(&root).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
}
