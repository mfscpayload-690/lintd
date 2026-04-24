# Contributing to lintd

Thank you for your interest in contributing. This document covers development setup, code conventions, and the pull request process.

## Code of conduct

Be respectful and constructive. Follow existing code style, write clear commit messages, and test your changes before submitting.

## Getting started

### Prerequisites

- Node.js 20+
- npm 10+
- Rust stable toolchain (`rustup toolchain install stable`)
- Linux desktop with WebKitGTK and GTK3 dev libraries

Ubuntu / Debian:
```bash
sudo apt-get install -y libwebkit2gtk-4.1-dev libgtk-3-dev \
  libayatana-appindicator3-dev librsvg2-dev patchelf build-essential
```

Arch Linux:
```bash
sudo pacman -S --needed webkit2gtk-4.1 gtk3 libayatana-appindicator librsvg base-devel
```

### Setup

```bash
git clone https://github.com/YOUR_USERNAME/lintd.git
cd lintd
npm install
npm run tauri dev
```

## Branch naming

| Type | Pattern |
|------|---------|
| Feature | `feature/short-description` |
| Bug fix | `fix/short-description` |
| Docs | `docs/what-you-changed` |
| Refactor | `refactor/component-name` |

## Code style

### TypeScript / React

- Strict TypeScript — no `any`
- Functional components and hooks only
- TanStack Query for all async data fetching
- Tailwind classes for styling — no inline styles except where dynamic values are required
- Export only what is needed; keep internals unexported

### Rust

- Run `cargo fmt` before committing
- Run `cargo clippy` and address all warnings
- Use `Result<T, E>` for error handling — no `unwrap()` in production paths
- Add doc comments (`///`) on all public functions and types

## Adding a new package manager backend

1. Create `src-tauri/src/pmal/your_manager.rs`
2. Implement the `PackageManager` trait:

```rust
use super::*;
use async_trait::async_trait;

pub struct YourManager;

#[async_trait]
impl PackageManager for YourManager {
    fn name(&self) -> &str { "your_manager" }
    fn source(&self) -> PackageSource { PackageSource::YourManager }
    fn detect(&self) -> bool {
        std::path::Path::new("/usr/bin/your-manager").exists()
    }
    async fn list_user_installed(&self) -> Result<Vec<Package>, PmalError> { todo!() }
    async fn list_orphans(&self) -> Result<Vec<Package>, PmalError> { Ok(vec![]) }
    async fn get_files(&self, name: &str) -> Result<Vec<String>, PmalError> { todo!() }
    async fn get_reverse_deps(&self, name: &str) -> Result<Vec<String>, PmalError> { todo!() }
    async fn remove(&self, name: &str, dry_run: bool) -> Result<RemovalResult, PmalError> { todo!() }
}
```

3. Add `YourManager` to the `PackageSource` enum in `pmal/mod.rs`
4. Register detection in `distro_detect.rs`
5. Add `'your_manager'` to the `PackageSource` union type in `src/types/lintd.ts`
6. Add entries to `sourceBadgeClassMap` and `sourceLabelMap` in `src/lib/presentation.ts`
7. Test package listing, orphan detection, removal preview, and actual removal

## Adding a distro logo

1. Add an SVG to `public/distro-logos/`
2. Add the distro ID mapping in `src/components/DistroLogo.tsx`
3. Add an `id_like` family fallback if needed
4. Run `npm run build` to verify

## Commit messages

Short one-liner, present tense, lowercase after the type prefix:

```
feat: add zypper package manager backend
fix: correct orphan detection for flatpak runtimes
style: unify source badge colors
chore: update dependencies
docs: update contributing guide
```

## Pull request process

1. Ensure `npx tsc --noEmit` and `cargo build` both pass
2. Keep PRs focused — one feature or fix per PR
3. Update documentation if you're changing behavior
4. Write a clear description: what, why, and how it was tested
5. Link related issues with `Fixes #123` or `Relates to #456`
6. Be responsive to review feedback

## Reporting bugs

Include:
- Distro and version
- Installed package managers (`which pacman apt dnf flatpak snap`)
- Steps to reproduce
- Expected vs actual behavior
- Error messages or terminal output

## Safety considerations

- Never modify the system-critical package block list without thorough review
- Always test removal flows with non-critical packages first
- Document any privilege escalation requirements
- Consider error cases and partial failure scenarios

## Questions

Open a GitHub issue with the `question` label, or review existing code for patterns and examples.
