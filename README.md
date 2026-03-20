# lintd

Cross-distro Linux package auditor built with Tauri, React, and Rust.

lintd provides a unified desktop interface for inspecting installed packages across multiple package managers, reviewing orphaned packages, previewing removal impact, and tracking removal history.

## Why this project exists

Linux package hygiene is fragmented across package ecosystems. Each distro family exposes different tools and output formats, making it hard to get one clear view of what is installed, what is unused, and what can be safely removed.

lintd addresses this by:

- Normalizing package metadata into a single application model
- Providing package safety checks before removal
- Recording removal history in a local database for traceability

## Current feature set

- System dashboard with distro, kernel, memory, storage, and package statistics
- Unified package list across supported sources
- Orphan package detection and bulk review flow
- Removal preview with reverse dependency checks, system-critical blocking, file deletion preview, and CLI command preview
- Removal history persisted in SQLite
- Light and dark theme support
- Linux packaging targets for AppImage and Deb

## Supported package sources

- pacman
- aur (detected via foreign pacman packages)
- apt
- dnf
- flatpak
- snap
- apk
- nix
- appimage (filesystem scan)

## Architecture overview

### Frontend

- React + TypeScript
- React Router for page navigation
- TanStack Query for async data fetching and cache invalidation
- Zustand for theme state
- Tailwind + Radix UI primitives for UI

### Backend

- Tauri v2 host application
- Rust command handlers exposed through Tauri invoke
- Package manager abstraction layer with one backend per source
- SQLite persistence using sqlx

### Data flow

1. Frontend calls typed command wrappers.
2. Tauri command handlers dispatch to matching backend manager.
3. Backend returns normalized models.
4. Frontend renders state and manages cache invalidation after mutations.

## Repository layout

```text
src/                     React frontend
	components/            shared UI and domain components
	pages/                 route-level pages
	lib/                   command wrappers, utilities, formatting, query keys
	types/                 shared frontend data contracts

src-tauri/src/           Rust backend and Tauri entrypoints
	pmal/                  package manager abstraction layer and source backends

.github/workflows/       CI workflows
```

## Prerequisites

- Node.js 20+
- npm 10+
- Rust stable toolchain
- Linux desktop environment with WebKitGTK and GTK3 dev libraries

Ubuntu or Debian example dependencies:

- libwebkit2gtk-4.1-dev
- libgtk-3-dev
- libayatana-appindicator3-dev
- librsvg2-dev
- patchelf

## Local development

Install dependencies:

```bash
npm install
```

Run web frontend only:

```bash
npm run dev
```

Run desktop app in development mode:

```bash
npm run tauri dev
```

Build frontend assets:

```bash
npm run build
```

Create desktop bundles:

```bash
npm run tauri build
```

## NPM scripts

| Script | Description |
| --- | --- |
| `dev` | Start the Vite development server |
| `build` | Run TypeScript checks and build frontend assets |
| `preview` | Preview the built frontend |
| `tauri` | Proxy to the Tauri CLI |

## Safety model

Package removal is guarded by conservative checks:

- hard block list for system-critical package names
- reverse dependency detection before allowing removal
- mandatory preview step before confirmation in UI

Note: package removal operations may invoke privileged commands via polkit, depending on the package source.

## Persistence

Removal records are stored in a local SQLite database under the user data directory and include:

- package name
- package source
- timestamp
- recovered space
- executed command preview

## CI and release workflow

GitHub Actions workflow builds Linux bundles on pushes to main and tagged releases:

- AppImage bundle
- Deb bundle

Tagged pushes matching v* generate a GitHub release with attached artifacts.

## Packaging notes

An AUR-style PKGBUILD is included under src-tauri for packaging workflows that require Arch-compatible metadata.

## Contributing

Contributions are welcome. If you plan to add a new package source backend:

1. Implement the PackageManager trait in a dedicated backend module.
2. Register detection and manager initialization in distro detection.
3. Ensure output is normalized to shared package types.
4. Validate removal safety logic and preview integrity.

## License

This repository is currently maintained without an explicit license file. Add a LICENSE file before distributing binaries broadly.
