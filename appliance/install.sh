#!/bin/bash
# Run from a trusted, extracted appliance release on Debian 13 x86-64.
set -euo pipefail
if [ "$#" -ne 2 ]; then
    echo "Usage: sudo $0 https://your-oneverse-api.example pulse-node-id" >&2
    exit 2
fi
[ "$(id -u)" = 0 ] || { echo 'Run the installer with sudo' >&2; exit 1; }
. /etc/os-release
[ "$ID" = debian ] && [ "$VERSION_ID" = 13 ] && [ "$(uname -m)" = x86_64 ] || {
    echo 'This release requires Debian 13 x86-64' >&2; exit 1;
}
api=$1
node_id=$2
case "$api" in https://*) ;; *) echo 'Use the HTTPS address of your Oneverse API' >&2; exit 2 ;; esac
[[ "$api" =~ ^https://[A-Za-z0-9.-]+(:[0-9]+)?/?$ ]] || { echo 'API address must be an HTTPS origin' >&2; exit 2; }
[[ "$node_id" =~ ^[A-Za-z0-9_-]{8,64}$ ]] || { echo 'Node id must be 8–64 letters, digits, - or _' >&2; exit 2; }
release=$(cd "$(dirname "$0")" && pwd)
cd "$release"
sha256sum --check SHA256SUMS
apt-get update
apt-get install -y --no-install-recommends ca-certificates python3 libstdc++6
python3 tools/engine-manifest.py engine --verify
revision=$(python3 -c 'import json; print(json.load(open("engine/manifest.json"))["source_revision"])')
if ! id pulse >/dev/null 2>&1; then
    useradd --system --create-home --home-dir /var/lib/oneverse-pulse --shell /usr/sbin/nologin pulse
fi
install -d -o pulse -g pulse -m 0700 /var/lib/oneverse-pulse /var/lib/oneverse-pulse/engine
install -d -m 0755 /etc/oneverse-pulse /opt/oneverse-pulse/engines
bundle=/opt/oneverse-pulse/engines/$revision
if [ -e "$bundle" ]; then
    cmp engine/manifest.json "$bundle/manifest.json" || { echo 'Existing revision differs; refusing replacement' >&2; exit 1; }
    python3 tools/engine-manifest.py "$bundle" --verify
else
    stage=$(mktemp -d /opt/oneverse-pulse/engines/.install.XXXXXX)
    trap 'rm -rf "$stage"' EXIT
    cp -a engine/. "$stage/"
    chown -R root:root "$stage"
    chmod -R u=rwX,go=rX "$stage"
    python3 tools/engine-manifest.py "$stage" --verify
    mv -T "$stage" "$bundle"
fi
# Preserve an existing configuration and paired identity on reinstallation.
if [ ! -e /etc/oneverse-pulse/pulsed.env ]; then
    umask 077
    cat > /etc/oneverse-pulse/pulsed.env <<EOF
PULSE_ENGINE_DIR=$bundle
ONEVERSE_API_URL=$api
PULSE_NODE_ID=$node_id
PULSE_DATA_DIR=/var/lib/oneverse-pulse/engine
PULSE_ROOM_PORT=8788
EOF
else
    echo 'Preserving /etc/oneverse-pulse/pulsed.env; update PULSE_ENGINE_DIR there to select a new revision.'
fi
systemctl stop oneverse-pulse.service || true
install -m 0755 bin/pulsed /usr/local/bin/pulsed
install -m 0644 systemd/oneverse-pulse.service /etc/systemd/system/
systemctl daemon-reload
systemctl enable --now oneverse-pulse.service
python3 - <<'PY'
import json
import time
import urllib.request
opener = urllib.request.build_opener(urllib.request.ProxyHandler({}))
for _ in range(60):
    try:
        with opener.open("http://127.0.0.1:8787/health", timeout=2) as response:
            health = json.load(response)
            if health["engine"]["state"] == "running":
                break
    except (OSError, ValueError, KeyError):
        pass
    time.sleep(1)
else:
    raise SystemExit("Engine not ready. Check pulsed.env and journalctl -u oneverse-pulse.service")
PY
echo 'Installed. Read the pairing code with: journalctl -u oneverse-pulse.service -f'
echo 'Pair this device in Oneverse → Screens → Pair a screen.'
echo 'Kiosk/HDMI installation remains optional; see the included Debian guide.'
