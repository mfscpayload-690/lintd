# Changelog

All notable changes to lintd are documented here.

## [1.1.0] - 2026-04-24

### Added
- Concurrent package scanning — all package managers now run in parallel, reducing scan time from sum(manager times) to max(manager times)
- Streaming scan with live progress — packages populate incrementally as each manager finishes, with a progress bar showing completed managers
- `useStreamingScan` React hook for accumulating partial scan results
- `scan_packages_streaming` Tauri command emitting per-manager `scan_progress` events
- Metric gauge circles on Dashboard for CPU, RAM, GPU VRAM, and storage mount points
- Package stats widget inline with system metrics gauges
- Package managers list in System Overview section
- Taskbar/dock icon now shows correctly on all Linux desktop environments
- `ScanProgressEvent` type for frontend/backend IPC

### Changed
- Dashboard layout redesigned — compact system overview, inline metric gauges, top packages as plain list
- Source badges unified to muted gray style (no more rainbow colors)
- Usage tags changed to text-only colored labels (no filled badge backgrounds)
- Sidebar narrowed to 200px, active route indicated by amber left border only
- Table rows densified to 32px height with monospace font for all data cells
- Card padding reduced from `p-6` to `p-3`
- Removal modal warning banners use left-border-only style
- `AppState` managers wrapped in `Arc` for concurrent access without holding the mutex
- `get_all_packages` and `get_orphans` rewritten to use concurrent helpers
- Linux-native warm color theme — off-white light mode, near-black dark mode, amber accent, `0.15rem` border radius

### Fixed
- Window icon not appearing in taskbar when running in dev mode or as AppImage

## [1.0.0] - 2026-03-21

Initial release.

- System dashboard with distro info, hardware stats, storage, and package statistics
- Unified package list across pacman, AUR, apt, dnf, flatpak, snap, apk, nix, and AppImage
- Orphan package detection with bulk removal flow
- Removal preview with reverse dependency checks, system-critical blocking, and file deletion preview
- Removal history persisted in SQLite
- Light and dark theme
- AppImage and Deb packaging targets
