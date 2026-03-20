use crate::pmal::{
    compute_usage_tag, get_desktop_atime, run_command, parse_stdout,
    Package, PackageManager, PackageSource, PmalError, RemovalResult, UsageTag,
};
use chrono::{DateTime, NaiveDateTime, Utc};

pub struct PacmanBackend;

impl PacmanBackend {
    pub fn new() -> Self {
        Self
    }

    fn has_aur_helper(&self) -> Option<String> {
        for helper in &["paru", "yay"] {
            if std::process::Command::new("which")
                .arg(helper)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
            {
                return Some(helper.to_string());
            }
        }
        None
    }

    fn parse_pacman_qi(output: &str) -> Vec<Package> {
        let mut packages = Vec::new();
        let mut current: Option<Package> = None;

        for block in output.split("\n\n") {
            let mut name = String::new();
            let mut version = String::new();
            let mut description = String::new();
            let mut size_bytes: u64 = 0;
            let mut install_date: Option<DateTime<Utc>> = None;

            for line in block.lines() {
                if let Some((key, value)) = line.split_once(':') {
                    let key = key.trim();
                    let value = value.trim();
                    match key {
                        "Name" => name = value.to_string(),
                        "Version" => version = value.to_string(),
                        "Description" => description = value.to_string(),
                        "Installed Size" => {
                            size_bytes = Self::parse_size(value);
                        }
                        "Install Date" => {
                            install_date = Self::parse_date(value);
                        }
                        _ => {}
                    }
                }
            }

            if !name.is_empty() {
                let last_used = get_desktop_atime(&name);
                let usage_tag = compute_usage_tag(last_used);
                packages.push(Package {
                    name,
                    version,
                    description,
                    install_date,
                    last_used,
                    size_bytes,
                    source: PackageSource::Pacman,
                    is_orphan: false,
                    usage_tag,
                    files: Vec::new(),
                });
            }
        }

        packages
    }

    fn parse_size(s: &str) -> u64 {
        let s = s.trim();
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() < 2 {
            return 0;
        }
        let num: f64 = parts[0].parse().unwrap_or(0.0);
        let unit = parts[1].to_uppercase();
        match unit.as_str() {
            "B" => num as u64,
            "KIB" | "KB" => (num * 1024.0) as u64,
            "MIB" | "MB" => (num * 1024.0 * 1024.0) as u64,
            "GIB" | "GB" => (num * 1024.0 * 1024.0 * 1024.0) as u64,
            _ => num as u64,
        }
    }

    fn parse_date(s: &str) -> Option<DateTime<Utc>> {
        let formats = [
            "%a %d %b %Y %I:%M:%S %p %Z",
            "%a %b %d %H:%M:%S %Y",
            "%Y-%m-%dT%H:%M:%S",
        ];
        for fmt in &formats {
            if let Ok(dt) = NaiveDateTime::parse_from_str(s.trim(), fmt) {
                return Some(dt.and_utc());
            }
        }
        None
    }
}

#[async_trait::async_trait]
impl PackageManager for PacmanBackend {
    fn name(&self) -> &str {
        "pacman"
    }

    fn source(&self) -> PackageSource {
        PackageSource::Pacman
    }

    fn detect(&self) -> bool {
        std::path::Path::new("/usr/bin/pacman").exists()
    }

    async fn list_user_installed(&self) -> Result<Vec<Package>, PmalError> {
        // Get explicitly installed packages
        let output = run_command("pacman", &["-Qe"]).await?;
        let stdout = parse_stdout(&output)?;
        let pkg_names: Vec<&str> = stdout.lines().filter_map(|l| l.split_whitespace().next()).collect();

        if pkg_names.is_empty() {
            return Ok(Vec::new());
        }

        // Get detailed info for all packages
        let mut args = vec!["-Qi"];
        args.extend(pkg_names.iter());
        let detail_output = run_command("pacman", &args).await?;
        let detail_stdout = parse_stdout(&detail_output)?;

        let mut packages = Self::parse_pacman_qi(&detail_stdout);

        // Mark AUR packages if we have an AUR helper
        if self.has_aur_helper().is_some() {
            let foreign_output = run_command("pacman", &["-Qm"]).await;
            if let Ok(foreign) = foreign_output {
                if let Ok(foreign_stdout) = parse_stdout(&foreign) {
                    let aur_names: Vec<String> = foreign_stdout
                        .lines()
                        .filter_map(|l| l.split_whitespace().next())
                        .map(String::from)
                        .collect();
                    for pkg in &mut packages {
                        if aur_names.contains(&pkg.name) {
                            pkg.source = PackageSource::Aur;
                        }
                    }
                }
            }
        }

        Ok(packages)
    }

    async fn list_orphans(&self) -> Result<Vec<Package>, PmalError> {
        let output = run_command("pacman", &["-Qdt"]).await?;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();

        if stdout.trim().is_empty() {
            return Ok(Vec::new());
        }

        let pkg_names: Vec<&str> = stdout.lines().filter_map(|l| l.split_whitespace().next()).collect();

        if pkg_names.is_empty() {
            return Ok(Vec::new());
        }

        let mut args = vec!["-Qi"];
        args.extend(pkg_names.iter());
        let detail_output = run_command("pacman", &args).await?;
        let detail_stdout = parse_stdout(&detail_output)?;

        let mut packages = Self::parse_pacman_qi(&detail_stdout);
        for pkg in &mut packages {
            pkg.is_orphan = true;
        }

        Ok(packages)
    }

    async fn get_reverse_deps(&self, pkg: &str) -> Result<Vec<String>, PmalError> {
        let output = run_command("pacman", &["-Qi", pkg]).await?;
        let stdout = parse_stdout(&output)?;

        for line in stdout.lines() {
            if let Some((key, value)) = line.split_once(':') {
                if key.trim() == "Required By" {
                    let deps: Vec<String> = value
                        .split_whitespace()
                        .filter(|s| *s != "None")
                        .map(String::from)
                        .collect();
                    return Ok(deps);
                }
            }
        }

        Ok(Vec::new())
    }

    async fn get_files(&self, pkg: &str) -> Result<Vec<String>, PmalError> {
        let output = run_command("pacman", &["-Ql", pkg]).await?;
        let stdout = parse_stdout(&output)?;

        let files: Vec<String> = stdout
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.splitn(2, ' ').collect();
                if parts.len() == 2 {
                    Some(parts[1].to_string())
                } else {
                    None
                }
            })
            .collect();

        Ok(files)
    }

    async fn remove(&self, pkg: &str, dry_run: bool) -> Result<RemovalResult, PmalError> {
        if dry_run {
            return Ok(RemovalResult {
                package_name: pkg.to_string(),
                success: true,
                message: format!("Dry run: would remove {}", pkg),
                space_recovered_bytes: 0,
            });
        }

        let output = run_command("pkexec", &["pacman", "-Rns", "--noconfirm", pkg]).await?;

        if output.status.success() {
            Ok(RemovalResult {
                package_name: pkg.to_string(),
                success: true,
                message: format!("Successfully removed {}", pkg),
                space_recovered_bytes: 0,
            })
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(PmalError::CommandFailed(stderr.to_string()))
        }
    }
}
