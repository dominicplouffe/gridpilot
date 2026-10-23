#!/usr/bin/env bash
# Build GridPilot.app (menu bar app, no Dock icon) into target/GridPilot.app.
# Run on macOS. The bundle gives GridPilot a stable identity, so the
# Accessibility permission is granted to "GridPilot" rather than to your
# terminal.
set -euo pipefail
cd "$(dirname "$0")/.."

VERSION=$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)
APP=target/GridPilot.app

cargo build --release -p gridpilot --target aarch64-apple-darwin
cargo build --release -p gridpilot --target x86_64-apple-darwin

rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
lipo -create -output "$APP/Contents/MacOS/gridpilot" \
  target/aarch64-apple-darwin/release/gridpilot \
  target/x86_64-apple-darwin/release/gridpilot

# The app runs `gridpilot run`; the same binary is the CLI.
cat > "$APP/Contents/MacOS/GridPilot" <<'SH'
#!/bin/sh
exec "$(dirname "$0")/gridpilot" run "$@"
SH
chmod +x "$APP/Contents/MacOS/GridPilot"

ICONSET=$(mktemp -d)/GridPilot.iconset
mkdir -p "$ICONSET"
for s in 16 32 128 256 512; do
  sips -z $s $s assets/brand/gridpilot-icon-1024.png --out "$ICONSET/icon_${s}x${s}.png" >/dev/null
  sips -z $((s * 2)) $((s * 2)) assets/brand/gridpilot-icon-1024.png --out "$ICONSET/icon_${s}x${s}@2x.png" >/dev/null
done
iconutil -c icns "$ICONSET" -o "$APP/Contents/Resources/GridPilot.icns"

cat > "$APP/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>GridPilot</string>
  <key>CFBundleDisplayName</key><string>GridPilot</string>
  <key>CFBundleIdentifier</key><string>com.dominicplouffe.gridpilot</string>
  <key>CFBundleVersion</key><string>${VERSION}</string>
  <key>CFBundleShortVersionString</key><string>${VERSION}</string>
  <key>CFBundleExecutable</key><string>GridPilot</string>
  <key>CFBundleIconFile</key><string>GridPilot</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>LSMinimumSystemVersion</key><string>12.0</string>
  <key>LSUIElement</key><true/>
  <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
PLIST

codesign --force --deep --sign - "$APP"
echo "Built $APP"
