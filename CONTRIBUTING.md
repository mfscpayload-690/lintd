# Contributing to lintd

Thank you for considering contributing to lintd! This document provides guidelines for contributing to the project.

## Code of Conduct

- Be respectful and constructive in discussions and code reviews
- Follow the existing code style and conventions
- Write clear commit messages and documentation
- Test your changes thoroughly before submitting

## Getting Started

### Prerequisites

Ensure you have all required dependencies installed:

- Node.js 20+
- npm 10+
- Rust stable toolchain
- Linux desktop environment with WebKitGTK and GTK3 dev libraries

Ubuntu/Debian example:
```bash
sudo apt-get install libwebkit2gtk-4.1-dev libgtk-3-dev \
  libayatana-appindicator3-dev librsvg2-dev patchelf
```

### Development Setup

1. Fork the repository
2. Clone your fork:
   ```bash
   git clone https://github.com/YOUR_USERNAME/lintd.git
   cd lintd
   ```
3. Install dependencies:
   ```bash
   npm install
   ```
4. Run the development server:
   ```bash
   npm run tauri dev
   ```

## Making Changes

### Branch Naming

Use descriptive branch names:
- `feature/package-manager-xyz` for new features
- `fix/issue-description` for bug fixes
- `docs/what-you-are-documenting` for documentation
- `refactor/component-name` for refactoring

### Code Style

#### Frontend (TypeScript/React)

- Use TypeScript strict mode
- Follow existing React patterns (functional components, hooks)
- Use TanStack Query for data fetching
- Keep components focused and single-purpose
- Use Tailwind classes for styling
- Add JSDoc comments for exported functions

Example:
```typescript
/**
 * Formats bytes into a human-readable string.
 */
export function formatBytes(bytes: number): string {
  // implementation
}
```

#### Backend (Rust)

- Run `cargo fmt` before committing
- Run `cargo clippy` and address warnings
- Follow Rust naming conventions (snake_case)
- Add doc comments for public APIs
- Use `Result<T, E>` for error handling
- Keep functions focused and testable

Example:
```rust
/// Lists all user-installed packages from this package manager.
///
/// # Errors
/// Returns `PmalError` if the command fails or times out.
pub async fn list_user_installed(&self) -> Result<Vec<Package>, PmalError> {
    // implementation
}
```

### Testing Your Changes

1. Run TypeScript type checking:
   ```bash
   npm run build
   ```

2. Test the desktop app:
   ```bash
   npm run tauri dev
   ```

3. Manually verify your changes work as expected

## Adding a New Package Manager Backend

To add support for a new package manager:

1. **Create a new backend module** in `src-tauri/src/pmal/`:
   ```bash
   touch src-tauri/src/pmal/your_manager.rs
   ```

2. **Implement the `PackageManager` trait**:
   ```rust
   use super::*;
   use async_trait::async_trait;

   pub struct YourManager;

   #[async_trait]
   impl PackageManager for YourManager {
       fn name(&self) -> &str { "your_manager" }

       fn source(&self) -> PackageSource {
           PackageSource::YourManager
       }

       fn detect(&self) -> bool {
           // Check if this package manager is available
           std::path::Path::new("/usr/bin/your-manager").exists()
       }

       async fn list_user_installed(&self) -> Result<Vec<Package>, PmalError> {
           // Implementation
       }

       // ... implement other trait methods
   }
   ```

3. **Add the source to the PackageSource enum** in `src-tauri/src/pmal/mod.rs`:
   ```rust
   pub enum PackageSource {
       // ... existing sources
       YourManager,
   }
   ```

4. **Register in distro detection** in `src-tauri/src/distro_detect.rs`:
   ```rust
   pub fn get_available_managers() -> Vec<Box<dyn PackageManager>> {
       let mut managers: Vec<Box<dyn PackageManager>> = vec![
           // ... existing managers
           Box::new(your_manager::YourManager),
       ];
       managers
   }
   ```

5. **Update the frontend type** in `src/types/lintd.ts`:
   ```typescript
   export type PackageSource =
     | 'pacman'
     | 'your_manager'
     // ... other sources
   ```

6. **Test thoroughly**:
   - Verify package detection works
   - Test orphan detection
   - Test removal preview
   - Test actual removal (with caution)

## Adding a New Distro Logo

1. Add SVG file to `public/distro-logos/`:
   ```bash
   cp your-logo.svg public/distro-logos/
   ```

2. Update `src/components/DistroLogo.tsx` with the distro ID mapping:
   ```typescript
   const DISTRO_LOGO_MAP: Record<string, string> = {
     // ... existing mappings
     'your-distro': 'your-logo.svg',
   };
   ```

3. If needed, add ID_LIKE fallback mapping for distro family support

## Commit Message Guidelines

Write clear, concise commit messages:

```
Brief description of change (50 chars or less)

More detailed explanation if needed. Explain what changed and why,
not how (the code shows how). Wrap at 72 characters.

Fixes #123
```

Examples:
- `Add support for Zypper package manager`
- `Fix orphan detection for Flatpak packages`
- `Improve error handling in removal preview`
- `Update README with troubleshooting section`

## Pull Request Process

1. **Ensure your code builds and runs** without errors
2. **Update documentation** if you're adding features or changing behavior
3. **Keep PRs focused** - one feature or fix per PR
4. **Write a clear PR description**:
   - What does this PR do?
   - Why is this change needed?
   - How has it been tested?
5. **Link related issues** using "Fixes #123" or "Relates to #456"
6. **Be responsive to feedback** and make requested changes promptly

## Reporting Bugs

When reporting bugs, include:

1. **Distro and version** (e.g., Arch Linux, Ubuntu 22.04)
2. **Installed package managers** (run `which pacman apt dnf flatpak snap` etc.)
3. **Steps to reproduce** the issue
4. **Expected vs actual behavior**
5. **Relevant error messages** or logs
6. **Screenshots** if applicable

## Suggesting Features

Before suggesting a new feature:

1. **Check existing issues** to avoid duplicates
2. **Explain the use case** - why is this feature valuable?
3. **Consider the scope** - does it fit the project's goals?
4. **Be specific** about what you're proposing

## Safety Considerations

When working on package removal functionality:

- **Never modify the system-critical package list** without thorough review
- **Always test with non-critical packages first**
- **Verify removal preview accuracy** before executing
- **Document any privilege escalation** requirements
- **Consider edge cases** and error scenarios

## Questions?

If you have questions about contributing:

1. Check the README for general information
2. Open a GitHub issue with the "question" label
3. Review existing code for examples and patterns

Thank you for contributing to lintd!
