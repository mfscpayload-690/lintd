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

## Screenshots
<img width="1920" height="1200" alt="Screenshot From 2026-03-21 11-54-21" src="https://github.com/user-attachments/assets/5f370f92-338c-4410-8eb9-7c1eac4a0d52" />
<img width="1920" height="1200" alt="Screenshot From 2026-03-21 11-54-34" src="https://github.com/user-attachments/assets/0c256721-9e5d-4994-95b8-210146b52577" />
<img width="1920" height="1200" alt="Screenshot From 2026-03-21 11-54-39" src="https://github.com/user-attachments/assets/daef2c2e-ecc1-41c6-b62e-b086ecc0087f" />
<img width="1920" height="1200" alt="Screenshot From 2026-03-21 11-54-48" src="https://github.com/user-attachments/assets/ec0a8144-6c50-4f98-97c1-cab4ac94e6af" />


## Installation

### From Release Binaries

Download the latest release from the [Releases page](https://github.com/mfscpayload-690/lintd/releases):

**AppImage:**
```bash
chmod +x lintd_*.AppImage
./lintd_*.AppImage
```

**Debian/Ubuntu (.deb):**
```bash
sudo dpkg -i lintd_*.deb
sudo apt-get install -f  # Install any missing dependencies
```

**Arch Linux (AUR):**
```bash
# Using yay or another AUR helper
yay -S lintd

# Or manually
git clone https://aur.archlinux.org/lintd.git
cd lintd
makepkg -si
```

### From Source

See the [Local development](#local-development) section below.

## Supported package sources

- **pacman** - Arch Linux native package manager
- **aur** - Arch User Repository (detected via foreign pacman packages)
- **apt** - Debian/Ubuntu package manager
- **dnf** - Fedora/RHEL package manager
- **flatpak** - Universal Linux application distribution
- **snap** - Canonical's universal package format
- **apk** - Alpine Linux package manager
- **nix** - Nix package manager
- **appimage** - Portable Linux applications (filesystem scan)

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

### For Running lintd

- Linux desktop environment with GTK3 and WebKitGTK support
- Optional: Package managers you want to audit (pacman, apt, dnf, flatpak, snap, etc.)

### For Development

- Node.js 20+
- npm 10+
- Rust stable toolchain
- Linux desktop environment with WebKitGTK and GTK3 dev libraries

#### Ubuntu/Debian Dependencies

```bash
sudo apt-get update
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  patchelf \
  build-essential \
  curl \
  wget \
  file
```

#### Arch Linux Dependencies

```bash
sudo pacman -S --needed \
  webkit2gtk-4.1 \
  gtk3 \
  libayatana-appindicator \
  librsvg \
  base-devel \
  curl \
  wget \
  file
```

#### Fedora/RHEL Dependencies

```bash
sudo dnf install -y \
  webkit2gtk4.1-devel \
  gtk3-devel \
  libappindicator-gtk3-devel \
  librsvg2-devel \
  openssl-devel \
  curl \
  wget \
  file
```

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

## Distro logo assets

Dashboard distro logos are loaded from local SVG assets in public/distro-logos.

To add support for another distro logo:

1. Add the logo file to public/distro-logos as an SVG.
2. Add an ID mapping in src/components/DistroLogo.tsx (exact distro ID).
3. If needed, add an ID_LIKE family fallback mapping in src/components/DistroLogo.tsx.
4. Rebuild and verify with npm run build.

## Troubleshooting

### App won't start

**Issue:** Double-clicking the AppImage does nothing or shows permission error.

**Solution:** Make the AppImage executable:
```bash
chmod +x lintd_*.AppImage
```

**Issue:** Error about missing libraries (GTK, WebKit, etc.)

**Solution:** Install the required system dependencies for your distro (see [Prerequisites](#prerequisites)).

### Package manager not detected

**Issue:** A package manager you have installed is not showing up in lintd.

**Solution:**
- Verify the package manager binary is in your PATH: `which pacman apt dnf flatpak snap`
- Restart lintd after installing a new package manager
- Check if the package manager requires additional setup (e.g., Nix requires sourcing profile)

### Cannot remove packages

**Issue:** Package removal fails with permission error.

**Solution:** lintd uses `pkexec` (polkit) for privilege escalation. Ensure:
- polkit is installed and running
- Your user is in the appropriate group (usually `wheel` or `sudo`)
- The package manager supports the removal command being used

**Issue:** Package shows as "system-critical" and cannot be removed.

**Solution:** This is intentional. lintd blocks removal of essential system packages (kernel, libc, init system, package managers, etc.) to prevent breaking your system. If you really need to remove such a package, use the native package manager directly.

### Orphan detection issues

**Issue:** No orphans are detected, but you expect some.

**Solution:**
- Orphan detection is package-manager specific
- Some managers (like Flatpak) don't have a concept of orphans
- Run your package manager's native orphan detection to compare (e.g., `pacman -Qdt`)

### Database errors

**Issue:** Error about SQLite database or removal history.

**Solution:** The database is stored in your user data directory. To reset it:
```bash
rm -rf ~/.local/share/lintd/
```
Note: This will delete your removal history.

### Build issues

**Issue:** `npm run tauri build` fails with Rust compilation errors.

**Solution:**
- Update Rust: `rustup update`
- Clean build artifacts: `cd src-tauri && cargo clean`
- Ensure all system dependencies are installed

**Issue:** Frontend build fails with TypeScript errors.

**Solution:**
- Delete `node_modules` and reinstall: `rm -rf node_modules && npm install`
- Clear TypeScript cache: `rm -rf node_modules/.cache`

### Performance issues

**Issue:** App is slow to load packages or appears frozen.

**Solution:**
- Initial package loading can take 10-30 seconds on systems with many packages
- If the app appears frozen for > 60 seconds, check terminal output for errors
- Large package lists (>5000 packages) may impact performance

### Still having issues?

Open an issue on [GitHub Issues](https://github.com/mfscpayload-690/lintd/issues) with:
- Your distro and version
- Installed package managers (`which pacman apt dnf flatpak snap`)
- Steps to reproduce
- Error messages or logs

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines on:

- Code style and conventions
- Development setup
- Adding new package manager backends
- Pull request process
- Reporting bugs and suggesting features

Quick contribution checklist:

1. Implement the PackageManager trait in a dedicated backend module
2. Register detection and manager initialization in distro detection
3. Ensure output is normalized to shared package types
4. Validate removal safety logic and preview integrity
5. Test thoroughly on the target platform

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
