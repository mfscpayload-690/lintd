//! Package Manager Abstraction Layer (PMAL)
//!
//! This module provides a unified interface for interacting with multiple Linux package managers.
//! Each package manager backend implements the `PackageManager` trait, allowing the application
//! to query packages, detect orphans, and execute removals in a consistent way.
//!
//! # Supported Package Managers
//!
//! - **pacman** - Arch Linux native package manager
//! - **apt** - Debian/Ubuntu package manager
//! - **dnf** - Fedora/RHEL package manager
//! - **flatpak** - Universal Linux application distribution
//! - **snap** - Canonical's universal package format
//! - **apk** - Alpine Linux package manager
//! - **nix** - Nix package manager
//! - **appimage** - Portable Linux applications (filesystem scan)
//!
//! # Architecture
//!
//! Each package manager backend is responsible for:
//! - Detecting if the package manager is available on the system
//! - Listing user-installed packages
//! - Detecting orphaned packages
//! - Fetching package metadata (files, dependencies)
//! - Generating removal previews
//! - Executing safe package removals
//!
//! All backends normalize their output to common data structures (`Package`, `RemovalPreview`, etc.)
//! so the frontend can work with a consistent API regardless of the underlying package manager.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::path::Path;
use std::sync::{Mutex, OnceLock};
use thiserror::Error;

pub mod apk;
pub mod appimage;
pub mod apt;
pub mod dnf;
pub mod flatpak;
pub mod nix;
pub mod pacman;
pub mod snap;

// ── Error types ──────────────────────────────────────────────────

#[derive(Error, Debug)]
pub enum PmalError {
    #[error("Command execution failed: {0}")]
    CommandFailed(String),

    #[error("Command timed out after {0} seconds")]
    Timeout(u64),

    #[error("Package not found: {0}")]
    PackageNotFound(String),

    #[error("Permission denied: {0}")]
    #[allow(dead_code)]
    PermissionDenied(String),

    #[error("Parse error: {0}")]
    #[allow(dead_code)]
    ParseError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

// ── Enums ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum PackageSource {
    Pacman,
    Aur,
    Apt,
    Dnf,
    Flatpak,
    Snap,
    AppImage,
    Apk,
    Nix,
    Manual,
}

impl fmt::Display for PackageSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PackageSource::Pacman => write!(f, "pacman"),
            PackageSource::Aur => write!(f, "aur"),
            PackageSource::Apt => write!(f, "apt"),
            PackageSource::Dnf => write!(f, "dnf"),
            PackageSource::Flatpak => write!(f, "flatpak"),
            PackageSource::Snap => write!(f, "snap"),
            PackageSource::AppImage => write!(f, "appimage"),
            PackageSource::Apk => write!(f, "apk"),
            PackageSource::Nix => write!(f, "nix"),
            PackageSource::Manual => write!(f, "manual"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UsageTag {
    Active,
    RarelyUsed,
    NeverLaunched,
}

// ── Package struct ───────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub description: String,
    pub install_date: Option<DateTime<Utc>>,
    pub last_used: Option<DateTime<Utc>>,
    pub size_bytes: u64,
    pub source: PackageSource,
    pub is_orphan: bool,
    pub usage_tag: UsageTag,
    #[serde(skip_serializing)]
    #[allow(dead_code)]
    pub files: Vec<String>,
}

// ── Removal types ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemovalPreview {
    pub package_name: String,
    pub description: String,
    pub files_to_delete: Vec<String>,
    pub reverse_deps: Vec<String>,
    pub is_system_critical: bool,
    pub size_to_recover_bytes: u64,
    pub cli_command_preview: String,
    pub safe_to_remove: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemovalResult {
    pub package_name: String,
    pub success: bool,
    pub message: String,
    pub space_recovered_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemovalRecord {
    pub id: i64,
    pub package_name: String,
    pub source: PackageSource,
    pub removed_at: DateTime<Utc>,
    pub space_recovered_bytes: u64,
    pub command_executed: String,
}

// ── System-critical packages (hard block list) ───────────────────

const SYSTEM_CRITICAL_PACKAGES: &[&str] = &[
    "linux",
    "linux-lts",
    "linux-zen",
    "linux-hardened",
    "linux-image",
    "linux-headers",
    "glibc",
    "musl",
    "libc6",
    "systemd",
    "openrc",
    "runit",
    "s6",
    "xorg-server",
    "wayland",
    "weston",
    "polkit",
    "dbus",
    "pacman",
    "apt",
    "dpkg",
    "dnf",
    "rpm",
    "apk-tools",
    "bash",
    "sh",
    "dash",
    "sudo",
];

pub fn is_system_critical(name: &str) -> bool {
    let lower = name.to_lowercase();
    SYSTEM_CRITICAL_PACKAGES.iter().any(|&critical| {
        lower == critical || lower.starts_with(&format!("{}-", critical))
    })
}

// ── PackageManager trait ─────────────────────────────────────────

/// Core trait that all package manager backends must implement.
///
/// This trait defines the contract for package manager implementations, providing
/// a uniform interface for package operations across different package ecosystems.
///
/// # Implementation Notes
///
/// - All async methods should have reasonable timeouts (default: 30s)
/// - Package lists should be normalized to the common `Package` struct
/// - Safety checks (system-critical packages, reverse deps) should be thorough
/// - Error messages should be descriptive for debugging
///
/// # Example
///
/// ```rust,ignore
/// pub struct MyPackageManager;
///
/// #[async_trait::async_trait]
/// impl PackageManager for MyPackageManager {
///     fn name(&self) -> &str { "my_manager" }
///     fn source(&self) -> PackageSource { PackageSource::Manual }
///     fn detect(&self) -> bool {
///         std::path::Path::new("/usr/bin/my-manager").exists()
///     }
///     // ... implement other methods
/// }
/// ```
#[async_trait::async_trait]
pub trait PackageManager: Send + Sync {
    /// Returns the human-readable name of this package manager.
    fn name(&self) -> &str;

    /// Returns the PackageSource enum variant for this manager.
    fn source(&self) -> PackageSource;

    /// Detects if this package manager is available on the system.
    ///
    /// Typically checks for the existence of the package manager's binary
    /// or configuration files.
    fn detect(&self) -> bool;

    /// Lists all user-installed packages from this package manager.
    ///
    /// # Returns
    ///
    /// A vector of `Package` structs with normalized metadata.
    ///
    /// # Errors
    ///
    /// Returns `PmalError` if the command fails or times out.
    async fn list_user_installed(&self) -> Result<Vec<Package>, PmalError>;

    /// Lists packages that are orphaned or no longer needed.
    ///
    /// Orphan detection logic is package-manager specific:
    /// - pacman: packages not required by any other package
    /// - apt: packages installed as dependencies that are no longer needed
    /// - Some managers may not support orphan detection
    ///
    /// # Returns
    ///
    /// A vector of orphaned `Package` structs.
    ///
    /// # Errors
    ///
    /// Returns `PmalError` if the command fails or times out.
    async fn list_orphans(&self) -> Result<Vec<Package>, PmalError>;

    /// Gets the list of packages that depend on this package.
    ///
    /// Used to prevent unsafe removals when other packages depend on this one.
    ///
    /// # Arguments
    ///
    /// * `pkg` - The package name to check
    ///
    /// # Returns
    ///
    /// A vector of package names that depend on `pkg`.
    ///
    /// # Errors
    ///
    /// Returns `PmalError` if the command fails or times out.
    async fn get_reverse_deps(&self, pkg: &str) -> Result<Vec<String>, PmalError>;

    /// Gets the list of files installed by this package.
    ///
    /// # Arguments
    ///
    /// * `pkg` - The package name to query
    ///
    /// # Returns
    ///
    /// A vector of absolute file paths installed by this package.
    ///
    /// # Errors
    ///
    /// Returns `PmalError` if the command fails or the package is not found.
    async fn get_files(&self, pkg: &str) -> Result<Vec<String>, PmalError>;

    /// Removes a package from the system.
    ///
    /// # Arguments
    ///
    /// * `pkg` - The package name to remove
    /// * `dry_run` - If true, simulates removal without actually removing
    ///
    /// # Returns
    ///
    /// A `RemovalResult` indicating success or failure.
    ///
    /// # Errors
    ///
    /// Returns `PmalError` if the removal fails or times out.
    ///
    /// # Safety
    ///
    /// This method should verify that the package is safe to remove before
    /// executing the removal command. It may invoke privileged commands via polkit.
    async fn remove(&self, pkg: &str, dry_run: bool) -> Result<RemovalResult, PmalError>;
}

// ── Subprocess helper ────────────────────────────────────────────

use std::process::Output;
use tokio::process::Command;

/// Runs a command with a 30-second timeout.
///
/// # Arguments
///
/// * `program` - The program to execute
/// * `args` - Command-line arguments to pass to the program
///
/// # Returns
///
/// The command output if successful.
///
/// # Errors
///
/// Returns `PmalError::Timeout` if the command takes longer than 30 seconds.
/// Returns `PmalError::CommandFailed` if the command fails to execute.
pub async fn run_command(program: &str, args: &[&str]) -> Result<Output, PmalError> {
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(30),
        Command::new(program).args(args).output(),
    )
    .await;

    match result {
        Ok(Ok(output)) => Ok(output),
        Ok(Err(e)) => Err(PmalError::CommandFailed(format!(
            "Failed to run {} {}: {}",
            program,
            args.join(" "),
            e
        ))),
        Err(_) => Err(PmalError::Timeout(30)),
    }
}

/// Parses command stdout, returning an error if the command failed.
///
/// # Arguments
///
/// * `output` - The command output to parse
///
/// # Returns
///
/// The stdout as a string if the command succeeded.
///
/// # Errors
///
/// Returns `PmalError::CommandFailed` with stderr content if the command failed.
pub fn parse_stdout(output: &Output) -> Result<String, PmalError> {
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(PmalError::CommandFailed(stderr.to_string()));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

// ── Usage tag helper ─────────────────────────────────────────────

/// Computes a usage tag based on when a package was last used.
///
/// # Arguments
///
/// * `last_used` - Optional timestamp of last usage
///
/// # Returns
///
/// - `UsageTag::NeverLaunched` if last_used is None
/// - `UsageTag::RarelyUsed` if last used more than 60 days ago
/// - `UsageTag::Active` if last used within 60 days
pub fn compute_usage_tag(last_used: Option<DateTime<Utc>>) -> UsageTag {
    match last_used {
        None => UsageTag::NeverLaunched,
        Some(ts) => {
            let days_ago = Utc::now().signed_duration_since(ts).num_days();
            if days_ago > 60 {
                UsageTag::RarelyUsed
            } else {
                UsageTag::Active
            }
        }
    }
}

fn is_always_active_package(pkg_name: &str) -> bool {
    let lower = pkg_name.to_lowercase();
    let patterns = [
        "lib",
        "headers",
        "dkms",
        "devel",
        "dev",
        "-git",
        "firmware",
        "driver",
        "kernel",
        "linux-",
        "codec",
        "runtime",
        "gtk",
        "qt",
    ];

    patterns.iter().any(|p| lower.contains(p))
}

fn push_name_variants(pkg_name: &str) -> Vec<String> {
    let mut variants = vec![pkg_name.to_string()];
    let underscore = pkg_name.replace('-', "_");
    if underscore != pkg_name {
        variants.push(underscore);
    }

    if let Some(first) = pkg_name.split('-').next() {
        if !first.is_empty() && !variants.iter().any(|v| v == first) {
            variants.push(first.to_string());
        }
    }

    variants
}

fn newer_atime_than_mtime(path: &Path) -> Option<DateTime<Utc>> {
    let metadata = std::fs::metadata(path).ok()?;
    let accessed = metadata.accessed().ok()?;
    let modified = metadata.modified().ok()?;
    if accessed <= modified {
        return None;
    }
    Some(accessed.into())
}

fn max_dt(current: Option<DateTime<Utc>>, candidate: Option<DateTime<Utc>>) -> Option<DateTime<Utc>> {
    match (current, candidate) {
        (None, None) => None,
        (Some(v), None) | (None, Some(v)) => Some(v),
        (Some(a), Some(b)) => Some(if a > b { a } else { b }),
    }
}

fn get_binary_atime(pkg_name: &str, files: &[String]) -> Option<DateTime<Utc>> {
    let bin_dirs = ["/usr/bin", "/usr/local/bin", "/usr/sbin", "/bin", "/sbin"];
    let variants = push_name_variants(pkg_name);
    let mut latest: Option<DateTime<Utc>> = None;

    for dir in &bin_dirs {
        for variant in &variants {
            let path = Path::new(dir).join(variant);
            latest = max_dt(latest, newer_atime_than_mtime(&path));
        }
    }

    for file in files {
        if file.starts_with("/usr/bin/")
            || file.starts_with("/usr/local/bin/")
            || file.starts_with("/usr/sbin/")
            || file.starts_with("/bin/")
            || file.starts_with("/sbin/")
        {
            latest = max_dt(latest, newer_atime_than_mtime(Path::new(file)));
        }
    }

    latest
}

fn parse_systemctl_timestamp(value: &str) -> Option<DateTime<Utc>> {
    let ts = value.trim();
    if ts.is_empty() || ts == "n/a" {
        return None;
    }

    chrono::DateTime::parse_from_str(ts, "%a %Y-%m-%d %H:%M:%S %Z")
        .or_else(|_| chrono::DateTime::parse_from_str(ts, "%a %Y-%m-%d %H:%M:%S %z"))
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

fn try_service_timestamp(service_name: &str) -> Option<DateTime<Utc>> {
    let output = std::process::Command::new("timeout")
        .args([
            "3s",
            "systemctl",
            "show",
            service_name,
            "--property=ActiveEnterTimestamp",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout.lines().next()?;
    let value = line.strip_prefix("ActiveEnterTimestamp=")?;
    parse_systemctl_timestamp(value)
}

fn get_service_last_active_time(pkg_name: &str) -> Option<DateTime<Utc>> {
    let mut names = vec![pkg_name.to_string(), format!("{}.service", pkg_name)];
    match pkg_name {
        "mariadb" => names.push("mariadb.service".to_string()),
        "mongodb" => {
            names.push("mongodb.service".to_string());
            names.push("mongod.service".to_string());
        }
        "postgresql" => names.push("postgresql.service".to_string()),
        "nginx" => names.push("nginx.service".to_string()),
        "sshd" => names.push("sshd.service".to_string()),
        _ => {}
    }

    let mut latest: Option<DateTime<Utc>> = None;
    for name in names {
        latest = max_dt(latest, try_service_timestamp(&name));
    }
    latest
}

fn history_contains_pkg(path: &Path, pkg_name: &str) -> bool {
    std::fs::read_to_string(path)
        .map(|contents| contents.to_lowercase().contains(&pkg_name.to_lowercase()))
        .unwrap_or(false)
}

fn fish_history_has_cmd(path: &Path, pkg_name: &str) -> bool {
    let contents = std::fs::read_to_string(path).unwrap_or_default();
    let needle = pkg_name.to_lowercase();
    contents
        .lines()
        .filter_map(|line| line.trim_start().strip_prefix("- cmd:"))
        .any(|cmd| cmd.to_lowercase().contains(&needle))
}

fn get_shell_history_recency(pkg_name: &str) -> Option<DateTime<Utc>> {
    let home = std::env::var("HOME").unwrap_or_default();
    if home.is_empty() {
        return None;
    }

    let bash = Path::new(&home).join(".bash_history");
    let zsh = Path::new(&home).join(".zsh_history");
    let fish_local = Path::new(&home).join(".local/share/fish/fish_history");
    let fish_config = Path::new(&home).join(".config/fish/fish_history");

    let mut latest: Option<DateTime<Utc>> = None;

    for path in [&bash, &zsh] {
        if history_contains_pkg(path, pkg_name) {
            if let Ok(metadata) = std::fs::metadata(path) {
                if let Ok(mtime) = metadata.modified() {
                    latest = max_dt(latest, Some(mtime.into()));
                }
            }
        }
    }

    for path in [&fish_local, &fish_config] {
        if fish_history_has_cmd(path, pkg_name) {
            latest = max_dt(latest, Some(Utc::now() - chrono::Duration::days(7)));
        } else if history_contains_pkg(path, pkg_name) {
            if let Ok(metadata) = std::fs::metadata(path) {
                if let Ok(mtime) = metadata.modified() {
                    latest = max_dt(latest, Some(mtime.into()));
                }
            }
        }
    }

    latest
}

fn get_desktop_atime(pkg_name: &str) -> Option<DateTime<Utc>> {
    let xdg_data_dirs = std::env::var("XDG_DATA_DIRS")
        .unwrap_or_else(|_| "/usr/share:/usr/local/share".to_string());
    let home = std::env::var("HOME").unwrap_or_default();

    let mut search_dirs: Vec<String> = vec![format!("{}/.local/share/applications", home)];
    for dir in xdg_data_dirs.split(':') {
        search_dirs.push(format!("{}/applications", dir));
    }

    for dir in &search_dirs {
        let desktop_file = format!("{}/{}.desktop", dir, pkg_name);
        if let Ok(metadata) = std::fs::metadata(&desktop_file) {
            if let Ok(accessed) = metadata.accessed() {
                let dt: DateTime<Utc> = accessed.into();
                return Some(dt);
            }
        }
    }
    None
}

static PACMAN_BIN_CACHE: OnceLock<Mutex<HashMap<String, Vec<String>>>> = OnceLock::new();
static PACMAN_AVAILABLE: OnceLock<bool> = OnceLock::new();

fn get_pacman_bin_paths_cached(pkg_name: &str) -> Vec<String> {
    let pacman_available = *PACMAN_AVAILABLE.get_or_init(|| Path::new("/usr/bin/pacman").exists());
    if !pacman_available {
        return Vec::new();
    }

    let cache = PACMAN_BIN_CACHE.get_or_init(|| Mutex::new(HashMap::new()));

    if let Ok(guard) = cache.lock() {
        if let Some(paths) = guard.get(pkg_name) {
            return paths.clone();
        }
    }

    let output = std::process::Command::new("pacman")
        .args(["-Ql", pkg_name])
        .output();

    let mut bin_paths = Vec::new();
    if let Ok(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.splitn(2, ' ').collect();
                if parts.len() == 2 {
                    let path = parts[1].trim();
                    if path.starts_with("/usr/bin/") {
                        bin_paths.push(path.to_string());
                    }
                }
            }
        }
    }

    if let Ok(mut guard) = cache.lock() {
        guard.insert(pkg_name.to_string(), bin_paths.clone());
    }

    bin_paths
}

/// Determines when a package was last used by checking multiple heuristics.
///
/// This function employs several strategies to determine package usage:
///
/// 1. **Binary access time** - Checks atime of executables in common bin directories
/// 2. **Systemd service timestamps** - For daemon packages, checks service last active time
/// 3. **Shell history** - Searches bash/zsh/fish history for package mentions
/// 4. **Desktop file access** - Checks .desktop file atime for GUI applications
/// 5. **Pacman file list cache** - Uses pacman's file list if available
///
/// # Arguments
///
/// * `pkg_name` - The package name to check
/// * `files` - Optional list of files installed by the package
///
/// # Returns
///
/// The most recent timestamp found across all heuristics, or None if no usage detected.
///
/// # Notes
///
/// - Packages matching common library/runtime patterns are always marked as active
/// - Access times may be unreliable on some filesystems (e.g., noatime mount option)
/// - This is a best-effort heuristic and may not be 100% accurate
pub fn get_last_used_time(pkg_name: &str, files: &[String]) -> Option<DateTime<Utc>> {
    if is_always_active_package(pkg_name) {
        return Some(Utc::now());
    }

    let mut latest: Option<DateTime<Utc>> = None;

    // SOURCE 1: Binary access time (+ file list hints)
    latest = max_dt(latest, get_binary_atime(pkg_name, files));

    // SOURCE 2: systemd service last active timestamp
    latest = max_dt(latest, get_service_last_active_time(pkg_name));

    // SOURCE 3: shell history recency
    latest = max_dt(latest, get_shell_history_recency(pkg_name));

    // SOURCE 4: desktop file access time
    latest = max_dt(latest, get_desktop_atime(pkg_name));

    // SOURCE 5: pacman binary list cache lookup
    let pacman_bin_paths = get_pacman_bin_paths_cached(pkg_name);
    latest = max_dt(latest, get_binary_atime(pkg_name, &pacman_bin_paths));

    latest
}
