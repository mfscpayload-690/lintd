use crate::pmal::{
    compute_usage_tag, get_desktop_atime, run_command, parse_stdout,
    Package, PackageManager, PackageSource, PmalError, RemovalResult,
};

pub struct ApkBackend;

impl ApkBackend {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl PackageManager for ApkBackend {
    fn name(&self) -> &str {
        "apk"
    }

    fn source(&self) -> PackageSource {
        PackageSource::Apk
    }

    fn detect(&self) -> bool {
        std::path::Path::new("/sbin/apk").exists()
            || std::path::Path::new("/usr/sbin/apk").exists()
    }

    async fn list_user_installed(&self) -> Result<Vec<Package>, PmalError> {
        // apk info -v gives name-version pairs
        let output = run_command("apk", &["info", "-v", "-s", "-d"]).await?;
        let stdout = parse_stdout(&output)?;

        let mut packages = Vec::new();
        let mut current_name = String::new();
        let mut current_version = String::new();
        let mut current_desc = String::new();
        let mut current_size: u64 = 0;

        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() {
                if !current_name.is_empty() {
                    let last_used = get_desktop_atime(&current_name);
                    let usage_tag = compute_usage_tag(last_used);
                    packages.push(Package {
                        name: current_name.clone(),
                        version: current_version.clone(),
                        description: current_desc.clone(),
                        install_date: None,
                        last_used,
                        size_bytes: current_size,
                        source: PackageSource::Apk,
                        is_orphan: false,
                        usage_tag,
                        files: Vec::new(),
                    });
                    current_name.clear();
                    current_version.clear();
                    current_desc.clear();
                    current_size = 0;
                }
                continue;
            }

            if line.contains(" description:") {
                current_desc = line.replacen(" description:", "", 1).trim().to_string();
            } else if line.contains(" installed size:") {
                let size_str = line.replacen(" installed size:", "", 1).trim().to_string();
                current_size = Self::parse_apk_size(&size_str);
            } else if current_name.is_empty() {
                // First non-empty line is name-version
                if let Some(idx) = line.rfind('-') {
                    current_name = line[..idx].to_string();
                    current_version = line[idx + 1..].to_string();
                } else {
                    current_name = line.to_string();
                }
            }
        }

        // Don't forget the last package
        if !current_name.is_empty() {
            let last_used = get_desktop_atime(&current_name);
            let usage_tag = compute_usage_tag(last_used);
            packages.push(Package {
                name: current_name,
                version: current_version,
                description: current_desc,
                install_date: None,
                last_used,
                size_bytes: current_size,
                source: PackageSource::Apk,
                is_orphan: false,
                usage_tag,
                files: Vec::new(),
            });
        }

        Ok(packages)
    }

    async fn list_orphans(&self) -> Result<Vec<Package>, PmalError> {
        // Packages installed as dependencies that are no longer needed
        let world_output = run_command("apk", &["info", "-r"]).await;
        // For Alpine, orphan detection is less straightforward
        // We look for packages not in the world file
        let world = std::fs::read_to_string("/etc/apk/world").unwrap_or_default();
        let world_pkgs: Vec<&str> = world.lines().map(|l| l.trim()).collect();

        let all = self.list_user_installed().await?;
        let mut orphans: Vec<Package> = all
            .into_iter()
            .filter(|p| !world_pkgs.contains(&p.name.as_str()))
            .map(|mut p| {
                p.is_orphan = true;
                p
            })
            .collect();

        Ok(orphans)
    }

    async fn get_reverse_deps(&self, pkg: &str) -> Result<Vec<String>, PmalError> {
        let output = run_command("apk", &["info", "-r", pkg]).await?;
        let stdout = parse_stdout(&output)?;

        let deps: Vec<String> = stdout
            .lines()
            .skip(1) // Skip the header line
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect();

        Ok(deps)
    }

    async fn get_files(&self, pkg: &str) -> Result<Vec<String>, PmalError> {
        let output = run_command("apk", &["info", "-L", pkg]).await?;
        let stdout = parse_stdout(&output)?;

        let files: Vec<String> = stdout
            .lines()
            .skip(1) // Skip header
            .map(|l| format!("/{}", l.trim()))
            .filter(|l| l.len() > 1)
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

        let output = run_command("pkexec", &["apk", "del", pkg]).await?;

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

impl ApkBackend {
    fn parse_apk_size(s: &str) -> u64 {
        let s = s.trim();
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.is_empty() {
            return 0;
        }
        let num: f64 = parts[0].parse().unwrap_or(0.0);
        let unit = if parts.len() > 1 {
            parts[1].to_uppercase()
        } else {
            "B".to_string()
        };
        match unit.as_str() {
            "KIB" | "KB" | "K" => (num * 1024.0) as u64,
            "MIB" | "MB" | "M" => (num * 1024.0 * 1024.0) as u64,
            "GIB" | "GB" | "G" => (num * 1024.0 * 1024.0 * 1024.0) as u64,
            _ => num as u64,
        }
    }
}
