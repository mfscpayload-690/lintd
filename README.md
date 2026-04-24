# lintd

Cross-distro Linux package auditor built with Tauri, React, and Rust.

lintd provides a unified desktop interface for inspecting installed packages across multiple package managers, reviewing orphaned packages, previewing removal impact, and tracking removal history — all in a dense, Linux-native UI.

## Why this project exists

Linux package hygiene is fragmented across ecosystems. Each distro family exposes different tools and output formats, making it hard to get one clear view of what is installed, what is unused, and what can be safely removed.

lintd addresses this by:

- Normalizing package metadata into a single application model
- Running all package manager queries concurrently for fast scans
- Streaming partial results to the UI as each manager finishes
- Providing safety checks before any removal
- Recording removal history in a local SQLite database

## Feature set

- **Dashboard** — system overview with distro info, CPU/RAM/GPU/storage gauges, package stats, and top packages by size
- **Packages** — unified package list across all detected sources with streaming scan progress, search, filter, and sort
- **Orphans** — orphan package detection with bulk review and removal flow
- **History** — removal history with space recovered tracking
- **Removal preview** — reverse dependency checks, system-critical blocking, file deletion preview, and CLI command preview
- **Light and dark theme** — warm off-white / near-black palette with amber accent
- **Linux packaging** — AppImage and Deb bundle targets

## Supported package sources

| Source | Description |
|--------|-------------|
| `pacman` | Arch Linux native package manager |
| `aur` | Arch User Repository (foreign pacman packages) |
| `apt` | Debian / Ubuntu package manager |
| `dnf` | Fedora / RHEL package manager |
| `flatpak` | Universal Linux application distribution |
| `snap` | Canonical universal package format |
| `apk` | Alpine Linux package manager |
| `nix` | Nix package manager |
| `appimage` | Portable Linux applications (filesystem scan) |

## Architecture

### Frontend

- React + TypeScript
- React Router for navigation
- TanStack Query for async data and cache invalidation
- Zustand for theme state
- Tailwind CSS + Radix UI primitives

### Backend

- Tauri v2 host application
- Rust async command handlers
- Package manager abstraction layer (`pmal/`) — one backend per source
- Concurrent scanning via `tokio::spawn` + `futures::join_all`
- Streaming scan events via Tauri event channel
- SQLite persistence via `sqlx`

### Data flow

```
Frontend → invoke() → Tauri command → pmal backend → normalized Package model
                                    ↓
                         emit("scan_progress") → useStreamingScan hook → UI
```

## Repository layout

```
src/
  components/       shared UI and domain components
  pages/            route-level pages (Dashboard, Packages, Orphans, History)
  lib/              command wrappers, hooks, utilities, query keys
  types/            shared TypeScript data contracts

src-tauri/src/
  pmal/             package manager abstraction layer and source backends
  commands.rs       Tauri command handlers
  lib.rs            app setup and window configuration
  sysinfo_collector.rs  system info collection
  distro_detect.rs  distro and package manager detection
  db.rs             SQLite persistence layer

.github/workflows/  CI and release workflows
public/distro-logos/ distro SVG logo assets
```

## Prerequisites

### Running lintd

- Linux desktop with GTK3 and WebKitGTK support
- Optional: any package managers you want to audit

### Development

- Node.js 20+
- npm 10+
- Rust stable toolchain
- WebKitGTK and GTK3 dev libraries

#### Ubuntu / Debian

```bash
sudo apt-get update
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  patchelf \
  build-essential \
  curl wget file
```

#### Arch Linux

```bash
sudo pacman -S --needed \
  webkit2gtk-4.1 gtk3 \
  libayatana-appindicator librsvg \
  base-devel curl wget file
```

#### Fedora / RHEL

```bash
sudo dnf install -y \
  webkit2gtk4.1-devel gtk3-devel \
  libappindicator-gtk3-devel librsvg2-devel \
  openssl-devel curl wget file
```

## Local development

```bash
# Install frontend dependencies
npm install

# Run desktop app in dev mode (hot reload)
npm run tauri dev

# Type-check frontend only
npx tsc --noEmit

# Build frontend assets
npm run build

# Build desktop bundles (AppImage + Deb)
npm run tauri build
```

## NPM scripts

| Script | Description |
|--------|-------------|
| `dev` | Start the Vite development server |
| `build` | TypeScript check + build frontend assets |
| `preview` | Preview the built frontend |
| `tauri` | Proxy to the Tauri CLI |

## Installation

### AppImage

```bash
chmod +x lintd_*.AppImage
./lintd_*.AppImage
```

### Debian / Ubuntu (.deb)

```bash
sudo dpkg -i lintd_*.deb
sudo apt-get install -f
```

### Arch Linux (AUR)

```bash
yay -S lintd
# or manually:
git clone https://aur.archlinux.org/lintd.git && cd lintd && makepkg -si
```

## Safety model

Package removal is guarded by:

- Hard block list for system-critical package names (kernel, libc, init, package managers)
- Reverse dependency detection before allowing removal
- Mandatory preview step before confirmation in the UI
- Privilege escalation via polkit / pkexec where required

## Persistence

Removal records are stored in a local SQLite database at `~/.local/share/lintd/` and include package name, source, timestamp, recovered space, and the executed command.

## Troubleshooting

**App won't start / permission error**
```bash
chmod +x lintd_*.AppImage
```

**Missing library error** — install system dependencies for your distro (see [Prerequisites](#prerequisites)).

**Package manager not detected** — verify the binary is in `$PATH` (`which pacman apt dnf flatpak snap`) and restart lintd.

**Cannot remove packages** — ensure polkit is installed and your user is in the `wheel` or `sudo` group.

**Package shows as system-critical** — this is intentional. Use the native package manager directly if you need to remove essential system packages.

**No orphans detected** — orphan detection is package-manager specific. Some managers (Flatpak, Snap) don't expose orphan information. Compare with `pacman -Qdt` or equivalent.

**Database errors** — reset the database:
```bash
rm -rf ~/.local/share/lintd/
```

**Build fails** — update Rust (`rustup update`), clean artifacts (`cargo clean` in `src-tauri/`), and ensure all system dependencies are installed.

**Still having issues?** Open an issue on [GitHub Issues](https://github.com/mfscpayload-690/lintd/issues) with your distro, installed package managers, steps to reproduce, and any error output.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup, code style, adding new package manager backends, and the pull request process.

## License

MIT — see [LICENSE](LICENSE).
