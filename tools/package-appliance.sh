#!/bin/bash
set -euo pipefail
if [ "$#" -ne 3 ]; then
    echo "Usage: $0 /path/to/Oneverse /path/to/new-release.tar.gz ONEVERSE_COMMIT_SHA" >&2
    exit 2
fi
source_dir=$(realpath "$1")
output=$(realpath -m "$2")
[ ! -e "$output" ] || { echo 'Output already exists' >&2; exit 1; }
root=$(cd "$(dirname "$0")/.." && pwd)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
release=$work/oneverse-pulse
mkdir -p "$release/bin" "$release/systemd" "$release/tools"
bash "$root/tools/package-engine.sh" "$source_dir" "$release/engine" "$3"
cargo build --manifest-path "$root/Cargo.toml" --locked --release -p pulsed -j "${CARGO_BUILD_JOBS:-4}"
install -m 0755 "$root/target/release/pulsed" "$release/bin/pulsed"
install -m 0755 "$root/appliance/install.sh" "$release/install.sh"
install -m 0644 "$root/tools/engine-manifest.py" "$release/tools/engine-manifest.py"
install -m 0644 "$root/appliance/systemd/oneverse-pulse.service" "$release/systemd/"
install -m 0644 "$root/appliance/debian/README.md" "$release/INSTALL.md"
install -m 0644 "$root/LICENSE" "$release/LICENSE"
(
    cd "$release"
    find . -type f ! -name SHA256SUMS -print0 | sort -z | xargs -0 sha256sum > SHA256SUMS
)
mkdir -p "$(dirname "$output")"
tar -C "$work" -czf "$output" oneverse-pulse
echo "Release archive: $output"
