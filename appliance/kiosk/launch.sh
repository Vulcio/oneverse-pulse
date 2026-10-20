#!/bin/sh
# Launch the real Songverse web screen in Chromium, or the local mock for development.
set -eu
SCREEN_URL="${SONGVERSE_SCREEN_URL:-https://songverse.one/screen}"
case "$SCREEN_URL" in
  http://127.0.0.1:*|http://localhost:*|https://*) ;;
  *) echo "Refusing unsupported screen URL: $SCREEN_URL" >&2; exit 2 ;;
esac
command -v cage >/dev/null || { echo "cage is required" >&2; exit 1; }
if command -v chromium >/dev/null; then
  BROWSER=chromium
elif command -v chromium-browser >/dev/null; then
  BROWSER=chromium-browser
else
  echo "Chromium is required" >&2; exit 1
fi
# Persistent browser profile retains Songverse screen pairing across restarts.
PROFILE="${SONGVERSE_CHROMIUM_PROFILE:-/var/lib/songverse/chromium}"
mkdir -p "$PROFILE"
exec cage -- "$BROWSER" \
  --kiosk \
  --no-first-run \
  --no-default-browser-check \
  --disable-session-crashed-bubble \
  --disable-features=Translate \
  --autoplay-policy=no-user-gesture-required \
  --user-data-dir="$PROFILE" \
  "$SCREEN_URL"
