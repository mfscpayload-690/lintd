#!/bin/bash
export APPIMAGE_EXTRACT_AND_RUN=1
export NO_STRIP=1
export DEPLOY_GTK_VERSION=3

# Override the gtk plugin with a no-op shim for Arch Linux
# GTK3 is always available as a system package on Arch
export LINUXDEPLOY_PLUGIN_GTK_PATH="$(pwd)/linuxdeploy-plugin-gtk.sh"

# Copy shim into tauri cache to override downloaded version
cp "$(pwd)/linuxdeploy-plugin-gtk.sh" \
	~/.cache/tauri/linuxdeploy-plugin-gtk.sh

# Backup original linuxdeploy and replace with our wrapper
if [ ! -f ~/.cache/tauri/linuxdeploy-x86_64.AppImage.orig ]; then
    cp ~/.cache/tauri/linuxdeploy-x86_64.AppImage ~/.cache/tauri/linuxdeploy-x86_64.AppImage.orig
fi
cp "$(pwd)/linuxdeploy-wrapper.sh" ~/.cache/tauri/linuxdeploy-x86_64.AppImage

npm run tauri build -- --bundles appimage

# Restore original linuxdeploy
if [ -f ~/.cache/tauri/linuxdeploy-x86_64.AppImage.orig ]; then
    mv ~/.cache/tauri/linuxdeploy-x86_64.AppImage.orig ~/.cache/tauri/linuxdeploy-x86_64.AppImage
fi
