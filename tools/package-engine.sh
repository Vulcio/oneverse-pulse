#!/bin/bash
# Produce a self-contained engine and console from one clean, pinned Oneverse checkout.
set -euo pipefail
if [ "$#" -ne 3 ]; then
    echo "Usage: $0 /path/to/Oneverse /path/to/new-bundle ONEVERSE_COMMIT_SHA" >&2
    exit 2
fi
source_dir=$(realpath "$1")
destination=$(realpath -m "$2")
revision=$3
tools_dir=$(cd "$(dirname "$0")" && pwd)
[ "$(uname -s)" = Linux ] || { echo 'Build on the target Linux architecture' >&2; exit 1; }
[ ! -e "$destination" ] || { echo 'Destination must not already exist' >&2; exit 1; }
check_source() {
    [ "$(git -C "$source_dir" rev-parse HEAD)" = "$revision" ] &&
    [ -z "$(git -C "$source_dir" status --porcelain --untracked-files=all)" ] || {
        echo 'Oneverse must be clean and at the requested full revision' >&2; exit 1;
    }
}
check_source
node_bin=$(realpath "${PULSE_NODE_BIN:-$(command -v node)}")
node_version=$("$node_bin" --version)
"$node_bin" -e 'const [a,b]=process.versions.node.split(".").map(Number); if(a<22 || (a===22 && b<18)) process.exit(1)'
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
cd "$source_dir"
CI=true pnpm install --frozen-lockfile
pnpm --filter @oneverse/core build
pnpm --filter @oneverse/pulse build
pnpm --filter @oneverse/screen-app build:pulse
# The root's TipTap patch belongs to web apps, absent from this production engine.
pnpm --filter @oneverse/pulse deploy --legacy --prod --frozen-lockfile --config.node-linker=hoisted --config.allow-unused-patches=true "$work/deploy"
check_source
python3 - "$work" "$source_dir" "$node_bin" <<'PY'
from pathlib import Path
import shutil
import sys
work, source, node = map(Path, sys.argv[1:])
out = work / "bundle"
(out / "runtime/bin").mkdir(parents=True)
shutil.copy2(node, out / "runtime/bin/node")
(out / "engine").mkdir()
for name in ("package.json", "dist", "node_modules"):
    src = work / "deploy" / name
    dst = out / "engine" / name
    if src.is_dir():
        # Flatten pnpm's internal links; bundles contain only regular files/directories.
        shutil.copytree(src, dst, symlinks=False)
    else:
        shutil.copy2(src, dst)
shutil.copytree(source / "apps/screen/dist/pulse", out / "console")
PY
# Node's combined licence includes its bundled third-party notices.
curl --proto '=https' --tlsv1.2 -fsSL "https://raw.githubusercontent.com/nodejs/node/$node_version/LICENSE" -o "$work/bundle/runtime/LICENSE"
python3 "$tools_dir/engine-manifest.py" "$work/bundle" --revision "$revision" --node-version "$node_version"
# Stage on the destination filesystem, publishing only the completed bundle.
mkdir -p "$(dirname "$destination")"
stage=$(mktemp -d "$(dirname "$destination")/.pulse-engine.XXXXXX")
trap 'rm -rf "$work" "$stage"' EXIT
cp -a "$work/bundle/." "$stage/"
mv -T "$stage" "$destination"
echo "Engine bundle: $destination (Oneverse $revision, Node $node_version)"
