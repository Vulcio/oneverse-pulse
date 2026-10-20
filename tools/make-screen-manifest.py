#!/usr/bin/env python3
"""Create a digest manifest for an already built local Songverse screen bundle.

This does not build the frontend and is not a signature: distribute/install via
a trusted channel. The Rust player rejects missing or changed files.
"""
import argparse
import hashlib
import json
from pathlib import Path

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("directory", type=Path)
    parser.add_argument("--revision", required=True, help="Songverse source commit SHA")
    args = parser.parse_args()
    root = args.directory.resolve(strict=True)
    if not (root / "index.html").is_file():
        parser.error("expected index.html in bundle root")
    files = {}
    for path in sorted(root.rglob("*")):
        if path.name == "manifest.json":
            continue
        if path.is_symlink():
            parser.error(f"symlink is not allowed: {path}")
        if path.is_file():
            rel = path.relative_to(root).as_posix()
            if rel != "index.html" and not rel.startswith("assets/"):
                parser.error(f"unsupported file path: {rel}")
            files[rel] = hashlib.sha256(path.read_bytes()).hexdigest()
    if not args.revision.strip():
        parser.error("--revision is required")
    (root / "manifest.json").write_text(json.dumps({
        "format": 1, "source_revision": args.revision, "files": files
    }, indent=2) + "\n", encoding="utf-8")
    print(f"Manifest written for {len(files)} files")
if __name__ == "__main__":
    main()
