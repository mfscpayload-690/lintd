use crate::pmal::{
    compute_usage_tag, get_desktop_atime, run_command, parse_stdout,
    Package, PackageManager, PackageSource, PmalError, RemovalResult,
};

pub struct NixBackend;

impl NixBackend {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl PackageManager for NixBackend {
    fn name(&self) -> &str {
        "nix"
    }

    fn source(&self) -> PackageSource {
        PackageSource::Nix
    }

    fn detect(&self) -> bool {
        std::path::Path::new("/run/current-system/sw/bin/nix-env").exists()
            || std::path::Path::new("/nix/var/nix/profiles").exists()
            || std::process::Command::new("which")
                .arg("nix-env")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
    }

    async fn list_user_installed(&self) -> Result<Vec<Package>, PmalError> {
        let output = run_command(
            "nix-env",
            &["--query", "--installed", "--json"],
        )
        .await?;
        let stdout = parse_stdout(&output)?;

        // Try JSON parsing first
        if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&stdout) {
            let mut packages = Vec::new();
            if let Some(obj) = json_val.as_object() {
                for (key, val) in obj {
                    let name = val
                        .get("pname")
                        .and_then(|v| v.as_str())
                        .unwrap_or(key)
                        .to_string();
                    let version = val
                        .get("version")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    let last_used = get_desktop_atime(&name);
                    let usage_tag = compute_usage_tag(last_used);

                    packages.push(Package {
                        name,
                        version,
                        description: String::new(),
                        install_date: None,
                        last_used,
                        size_bytes: 0,
                        source: PackageSource::Nix,
                        is_orphan: false,
                        usage_tag,
                        files: Vec::new(),
                    });
                }
            }
            return Ok(packages);
        }

        // Fallback to plain text parsing
        let output = run_command(
            "nix-env",
            &["--query", "--installed", "--attr-path", "--no-name", "--out-path"],
        )
        .await?;
        let stdout = parse_stdout(&output)?;

        let mut packages = Vec::new();
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            let full_name = parts[0];
            let (name, version) = if let Some(idx) = full_name.rfind('-') {
                (full_name[..idx].to_string(), full_name[idx + 1..].to_string())
            } else {
                (full_name.to_string(), String::new())
            };

            let last_used = get_desktop_atime(&name);
            let usage_tag = compute_usage_tag(last_used);

            packages.push(Package {
                name,
                version,
                description: String::new(),
                install_date: None,
                last_used,
                size_bytes: 0,
                source: PackageSource::Nix,
                is_orphan: false,
                usage_tag,
                files: Vec::new(),
            });
        }

        Ok(packages)
    }

    async fn list_orphans(&self) -> Result<Vec<Package>, PmalError> {
        // nix-store --gc --print-dead shows store paths that would be collected
        let output = run_command("nix-store", &["--gc", "--print-dead"]).await?;
        let stdout = parse_stdout(&output)?;

        let mut packages = Vec::new();
        for line in stdout.lines() {
            let path = line.trim();
            if path.is_empty() || !path.starts_with("/nix/store/") {
                continue;
            }

            // Extract name from nix store path: /nix/store/hash-name-version
            let store_name = path.strip_prefix("/nix/store/").unwrap_or(path);
            let after_hash = if let Some(idx) = store_name.find('-') {
                &store_name[idx + 1..]
            } else {
                store_name
            };

            let (name, version) = if let Some(idx) = after_hash.rfind('-') {
                (after_hash[..idx].to_string(), after_hash[idx + 1..].to_string())
            } else {
                (after_hash.to_string(), String::new())
            };

            // Get size of the store path
            let size_bytes = std::fs::metadata(path)
                .map(|m| m.len())
                .unwrap_or(0);

            packages.push(Package {
                name,
                version,
                description: format!("Dead nix store path: {}", path),
                install_date: None,
                last_used: None,
                size_bytes,
                source: PackageSource::Nix,
                is_orphan: true,
                usage_tag: crate::pmal::UsageTag::NeverLaunched,
                files: vec![path.to_string()],
            });
        }

        Ok(packages)
    }

    async fn get_reverse_deps(&self, pkg: &str) -> Result<Vec<String>, PmalError> {
        let output = run_command("nix-store", &["--query", "--referrers", pkg]).await;

        match output {
            Ok(o) => {
                let stdout = parse_stdout(&o)?;
                let deps: Vec<String> = stdout
                    .lines()
                    .map(|l| l.trim().to_string())
                    .filter(|l| !l.is_empty())
                    .collect();
                Ok(deps)
            }
            Err(_) => Ok(Vec::new()),
        }
    }

    async fn get_files(&self, pkg: &str) -> Result<Vec<String>, PmalError> {
        // For nix, the "files" are the store path contents
        let output = run_command("nix-store", &["--query", "--outputs", pkg]).await;
        match output {
            Ok(o) => {
                let stdout = parse_stdout(&o)?;
                let files: Vec<String> = stdout
                    .lines()
                    .map(|l| l.trim().to_string())
                    .filter(|l| !l.is_empty())
                    .collect();
                Ok(files)
            }
            Err(_) => Ok(Vec::new()),
        }
    }

    async fn remove(&self, pkg: &str, dry_run: bool) -> Result<RemovalResult, PmalError> {
        if dry_run {
            return Ok(RemovalResult {
                package_name: pkg.to_string(),
                success: true,
                message: format!("Dry run: would remove {} via nix-env -e", pkg),
                space_recovered_bytes: 0,
            });
        }

        let output = run_command("nix-env", &["--uninstall", pkg]).await?;

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
