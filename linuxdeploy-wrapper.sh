#\!/bin/bash
# Wrapper for linuxdeploy that disables stripping on Arch Linux
# to avoid compatibility issues with .relr.dyn sections

export NO_STRIP=1
exec ~/.cache/tauri/linuxdeploy-x86_64.AppImage.orig "$@"
