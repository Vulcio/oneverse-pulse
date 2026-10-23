#!/usr/bin/env python3
"""Create/check an engine manifest. Integrity only; install through a trusted channel."""
import argparse
import hashlib
import json
from pathlib import Path
import re


def files(root):
    result = {}
    for path in sorted(root.rglob("*")):
        if path.is_symlink():
            raise ValueError(f"symlink in engine bundle: {path}")
        if path.is_dir():
            continue
        if not path.is_file():
            raise ValueError(f"special file in engine bundle: {path}")
        relative = path.relative_to(root).as_posix()
        if relative == "manifest.json":
            continue
        with path.open("rb") as stream:
            result[relative] = hashlib.file_digest(stream, "sha256").hexdigest()
    if len(result) > 20_000:
        raise ValueError("too many files in engine bundle")
    for required in ("runtime/bin/node", "engine/dist/main.js", "console/pulse.html"):
        if required not in result:
            raise ValueError(f"missing {required}")
    return result


def verify(root):
    manifest = json.loads((root / "manifest.json").read_text())
    if manifest["format"] != 1 or not re.fullmatch(r"[0-9a-f]{40}", manifest["source_revision"]):
        raise ValueError("invalid engine manifest")
    if files(root) != manifest["files"]:
        raise ValueError("engine bundle checksum or file list mismatch")
    return manifest


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    parser.add_argument("--verify", action="store_true")
    parser.add_argument("--revision")
    parser.add_argument("--node-version")
    args = parser.parse_args()
    root = args.directory.resolve(strict=True)
    if args.verify:
        verify(root)
    else:
        if not args.revision or not re.fullmatch(r"[0-9a-f]{40}", args.revision):
            parser.error("a full --revision is required")
        if not args.node_version or not re.fullmatch(r"v\d+\.\d+\.\d+", args.node_version):
            parser.error("--node-version must be an exact Node release")
        import platform
        manifest = {
            "format": 1, "source_revision": args.revision,
            "node_version": args.node_version, "platform": "linux",
            "architecture": platform.machine(), "files": files(root),
        }
        (root / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    print("Engine bundle integrity verified")


if __name__ == "__main__":
    main()
