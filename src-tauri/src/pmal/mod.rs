use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
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
    PermissionDenied(String),

    #[error("Parse error: {0}")]
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

#[async_trait::async_trait]
pub trait PackageManager: Send + Sync {
    fn name(&self) -> &str;
    fn source(&self) -> PackageSource;
    fn detect(&self) -> bool;
    async fn list_user_installed(&self) -> Result<Vec<Package>, PmalError>;
    async fn list_orphans(&self) -> Result<Vec<Package>, PmalError>;
    async fn get_reverse_deps(&self, pkg: &str) -> Result<Vec<String>, PmalError>;
    async fn get_files(&self, pkg: &str) -> Result<Vec<String>, PmalError>;
    async fn remove(&self, pkg: &str, dry_run: bool) -> Result<RemovalResult, PmalError>;
}

// ── Subprocess helper ────────────────────────────────────────────

use std::process::Output;
use tokio::process::Command;

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

pub fn parse_stdout(output: &Output) -> Result<String, PmalError> {
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(PmalError::CommandFailed(stderr.to_string()));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

// ── Usage tag helper ─────────────────────────────────────────────

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

/// Try to find last access time from a .desktop file for the given package name.
pub fn get_desktop_atime(pkg_name: &str) -> Option<DateTime<Utc>> {
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
