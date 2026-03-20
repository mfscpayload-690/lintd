use crate::pmal::{
    compute_usage_tag, get_desktop_atime, run_command, parse_stdout,
    Package, PackageManager, PackageSource, PmalError, RemovalResult,
};

pub struct SnapBackend;

impl SnapBackend {
    pub fn new() -> Self { Self }

    async fn get_snap_size(name: &str) -> u64 {
        if let Ok(o) = run_command("snap", &["info", name]).await {
            if let Ok(stdout) = parse_stdout(&o) {
                for line in stdout.lines() {
                    if line.trim_start().starts_with("installed:") {
                        if let Some(size_part) = line.split('(').nth(1) {
                            if let Some(size_str) = size_part.split(')').next() {
                                return Self::parse_size(size_str);
                            }
                        }
                    }
                }
            }
        }
        0
    }

    async fn get_snap_description(name: &str) -> String {
        if let Ok(o) = run_command("snap", &["info", name]).await {
            if let Ok(stdout) = parse_stdout(&o) {
                for line in stdout.lines() {
                    if line.trim_start().starts_with("summary:") {
                        return line.split_once(':').map(|(_, v)| v.trim().to_string()).unwrap_or_default();
                    }
                }
            }
        }
        String::new()
    }

    fn parse_size(s: &str) -> u64 {
        let parts: Vec<&str> = s.trim().split_whitespace().collect();
        if parts.is_empty() { return 0; }
        let num: f64 = parts[0].parse().unwrap_or(0.0);
        let unit = if parts.len() > 1 { parts[1].to_uppercase() } else { "B".into() };
        match unit.as_str() {
            "KB" | "KIB" | "K" => (num * 1024.0) as u64,
            "MB" | "MIB" | "M" => (num * 1048576.0) as u64,
            "GB" | "GIB" | "G" => (num * 1073741824.0) as u64,
            _ => num as u64,
        }
    }
}

#[async_trait::async_trait]
impl PackageManager for SnapBackend {
    fn name(&self) -> &str { "snap" }
    fn source(&self) -> PackageSource { PackageSource::Snap }

    fn detect(&self) -> bool {
        std::path::Path::new("/usr/bin/snap").exists()
    }

    async fn list_user_installed(&self) -> Result<Vec<Package>, PmalError> {
        let output = run_command("snap", &["list"]).await?;
        let stdout = parse_stdout(&output)?;

        let mut packages = Vec::new();
        for line in stdout.lines().skip(1) {
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() < 4 { continue; }
            let name = fields[0].to_string();
            let version = fields[1].to_string();
            let notes = if fields.len() > 5 { fields[5] } else { "" };
            if notes.contains("disabled") { continue; }

            let size_bytes = Self::get_snap_size(&name).await;
            let description = Self::get_snap_description(&name).await;
            let last_used = get_desktop_atime(&name);
            let usage_tag = compute_usage_tag(last_used);

            packages.push(Package {
                name, version, description, install_date: None, last_used,
                size_bytes, source: PackageSource::Snap, is_orphan: false,
                usage_tag, files: Vec::new(),
            });
        }
        Ok(packages)
    }

    async fn list_orphans(&self) -> Result<Vec<Package>, PmalError> {
        let output = run_command("snap", &["list", "--all"]).await?;
        let stdout = parse_stdout(&output)?;
        let mut orphans = Vec::new();
        for line in stdout.lines().skip(1) {
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() < 5 { continue; }
            let notes = if fields.len() > 5 { fields[5] } else { "" };
            if !notes.contains("disabled") { continue; }

            let name = fields[0].to_string();
            let version = fields[1].to_string();
            let size_bytes = Self::get_snap_size(&name).await;
            orphans.push(Package {
                name, version, description: "Disabled snap (old revision)".into(),
                install_date: None, last_used: None, size_bytes,
                source: PackageSource::Snap, is_orphan: true,
                usage_tag: crate::pmal::UsageTag::NeverLaunched, files: Vec::new(),
            });
        }
        Ok(orphans)
    }

    async fn get_reverse_deps(&self, _pkg: &str) -> Result<Vec<String>, PmalError> {
        Ok(Vec::new()) // Snaps are self-contained
    }

    async fn get_files(&self, pkg: &str) -> Result<Vec<String>, PmalError> {
        let mut files = Vec::new();
        let snap_path = format!("/snap/{}", pkg);
        if std::path::Path::new(&snap_path).exists() { files.push(snap_path); }
        let home = std::env::var("HOME").unwrap_or_default();
        let snap_data = format!("{}/snap/{}", home, pkg);
        if std::path::Path::new(&snap_data).exists() { files.push(snap_data); }
        Ok(files)
    }

    async fn remove(&self, pkg: &str, dry_run: bool) -> Result<RemovalResult, PmalError> {
        if dry_run {
            return Ok(RemovalResult { package_name: pkg.into(), success: true,
                message: format!("Dry run: would remove {} via snap remove", pkg),
                space_recovered_bytes: 0 });
        }
        let output = run_command("pkexec", &["snap", "remove", pkg]).await?;
        if output.status.success() {
            Ok(RemovalResult { package_name: pkg.into(), success: true,
                message: format!("Successfully removed {}", pkg), space_recovered_bytes: 0 })
        } else {
            Err(PmalError::CommandFailed(String::from_utf8_lossy(&output.stderr).into()))
        }
    }
}
