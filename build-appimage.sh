#!/bin/bash

# AppImage Build Script for Arch Linux
# This script works around the linuxdeploy stripping issue with .relr.dyn sections
# by building the AppDir manually and then creating the AppImage with NO_STRIP=1

set -e

echo "Building lintd AppImage for Arch Linux..."

# Build the project first
npm run build
cd src-tauri
cargo build --release
cd ..

echo "Creating AppDir manually..."

# Set up build directory
BUILD_DIR="src-tauri/target/release/bundle/appimage"
APPDIR="$BUILD_DIR/Lintd.AppDir"
rm -rf "$BUILD_DIR"
mkdir -p "$APPDIR"

# Build with deb first to get proper tauri bundling setup
npm run tauri build -- --bundles deb

# Copy the binary
mkdir -p "$APPDIR/usr/bin"
cp src-tauri/target/release/lintd "$APPDIR/usr/bin/"

# Copy desktop file and icon
mkdir -p "$APPDIR/usr/share/applications"
mkdir -p "$APPDIR/usr/share/icons/hicolor/128x128/apps"
cp src-tauri/lintd.desktop "$APPDIR/usr/share/applications/"
cp src-tauri/icons/128x128.png "$APPDIR/usr/share/icons/hicolor/128x128/apps/Lintd.png"
cp src-tauri/icons/128x128.png "$APPDIR/Lintd.png"
ln -sf Lintd.png "$APPDIR/.DirIcon"
ln -sf usr/share/applications/lintd.desktop "$APPDIR/Lintd.desktop"

# Create AppRun
cat > "$APPDIR/AppRun" << 'APPRUN_SCRIPT'
#!/bin/sh
SELF=$(readlink -f "$0")
HERE=${SELF%/*}
export PATH="${HERE}/usr/bin/:${PATH}"
export LD_LIBRARY_PATH="${HERE}/usr/lib/:${LD_LIBRARY_PATH}"
export XDG_DATA_DIRS="${HERE}/usr/share/:${XDG_DATA_DIRS}"
cd "${HERE}"
exec "${HERE}/usr/bin/lintd" "$@"
APPRUN_SCRIPT
chmod +x "$APPDIR/AppRun"

echo "Running linuxdeploy with NO_STRIP..."

# Use linuxdeploy to bundle dependencies and create the AppImage
cd "$BUILD_DIR"
NO_STRIP=1 ~/.cache/tauri/linuxdeploy-x86_64.AppImage --appdir Lintd.AppDir --output appimage

echo "AppImage build completed successfully!"
ls -la *.AppImage