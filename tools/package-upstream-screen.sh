#!/bin/sh
# Package Songverse's dedicated, client-only ScreenStage build for the Rust player.
# Run this script from oneverse-pulse after building the upstream web workspace.
set -eu
if [ "$#" -ne 3 ]; then
  echo "Usage: $0 /path/to/Songverse /path/to/bundle ONEVERSE_COMMIT_SHA" >&2
  exit 2
fi
SOURCE=$1
DEST=$2
REVISION=$3
case "$REVISION" in
  *[!0-9a-fA-F]*|"") echo "Source revision must be a hexadecimal git commit SHA" >&2; exit 2 ;;
esac
if [ "${#REVISION}" -ne 40 ]; then
  echo "Source revision must be a full 40-character SHA" >&2; exit 2
fi
BUILT="$SOURCE/apps/web/dist/player-screen"
if [ ! -f "$BUILT/player-screen.html" ] || [ ! -d "$BUILT/assets" ]; then
  echo "Missing standalone screen build. In Songverse/apps/web run: vp build --config vite.player.config.ts" >&2
  exit 1
fi
if [ -e "$DEST" ]; then
  echo "Destination must not already exist: $DEST" >&2
  exit 1
fi
mkdir -p "$DEST/assets"
cp "$BUILT/player-screen.html" "$DEST/index.html"
cp -R "$BUILT/assets/." "$DEST/assets/"
python3 "$(dirname "$0")/make-screen-manifest.py" "$DEST" --revision "$REVISION"
echo "Bundle staged at $DEST"
echo "Run pulsed --dev with PULSE_BUNDLE_DIR=$DEST and point Chromium to http://127.0.0.1:8787/offline/screen"
