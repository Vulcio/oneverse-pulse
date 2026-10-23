#!/bin/sh
# Launch Oneverse's real web screen, or the local mock with pulsed --dev.
set -eu
# Compatibility for one release. A present PULSE_* value always takes precedence.
if [ "${SONGVERSE_SCREEN_URL+x}" = x ]; then
  echo "Warning: SONGVERSE_SCREEN_URL is deprecated; use PULSE_SCREEN_URL (legacy support lasts one release)." >&2
fi
if [ "${SONGVERSE_CHROMIUM_PROFILE+x}" = x ]; then
  echo "Warning: SONGVERSE_CHROMIUM_PROFILE is deprecated; use PULSE_CHROMIUM_PROFILE (legacy support lasts one release)." >&2
fi
SCREEN_URL="${PULSE_SCREEN_URL-${SONGVERSE_SCREEN_URL-https://screen.oneverse.top}}"
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
PROFILE="${PULSE_CHROMIUM_PROFILE-${SONGVERSE_CHROMIUM_PROFILE-/var/lib/oneverse-pulse/chromium}}"
[ -n "$PROFILE" ] || { echo "Chromium profile must not be empty" >&2; exit 2; }
umask 077
mkdir -p "$PROFILE"
chmod 0700 "$PROFILE"
exec cage -- "$BROWSER" \
  --kiosk \
  --no-first-run \
  --no-default-browser-check \
  --disable-session-crashed-bubble \
  --disable-features=Translate \
  --autoplay-policy=no-user-gesture-required \
  --user-data-dir="$PROFILE" \
  "$SCREEN_URL"
