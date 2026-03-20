use crate::pmal::{
    compute_usage_tag, get_desktop_atime, run_command, parse_stdout,
    Package, PackageManager, PackageSource, PmalError, RemovalResult,
};
use chrono::{DateTime, NaiveDateTime, Utc};

pub struct FlatpakBackend;

impl FlatpakBackend {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl PackageManager for FlatpakBackend {
    fn name(&self) -> &str {
        "flatpak"
    }

    fn source(&self) -> PackageSource {
        PackageSource::Flatpak
    }

    fn detect(&self) -> bool {
        std::path::Path::new("/usr/bin/flatpak").exists()
    }

    async fn list_user_installed(&self) -> Result<Vec<Package>, PmalError> {
        let output = run_command(
            "flatpak",
            &[
                "list",
                "--app",
                "--columns=application,name,version,size,description,installation",
            ],
        )
        .await?;
        let stdout = parse_stdout(&output)?;

        let mut packages = Vec::new();
        for line in stdout.lines() {
            let fields: Vec<&str> = line.split('\t').collect();
            if fields.len() < 4 {
                continue;
            }

            let app_id = fields[0].trim().to_string();
            let name = if fields.len() > 1 && !fields[1].trim().is_empty() {
                fields[1].trim().to_string()
            } else {
                app_id.clone()
            };
            let version = if fields.len() > 2 {
                fields[2].trim().to_string()
            } else {
                String::new()
            };
            let size_bytes = if fields.len() > 3 {
                Self::parse_flatpak_size(fields[3].trim())
            } else {
                0
            };
            let description = if fields.len() > 4 {
                fields[4].trim().to_string()
            } else {
                String::new()
            };

            let last_used = get_desktop_atime(&app_id)
                .or_else(|| get_desktop_atime(&name));
            let usage_tag = compute_usage_tag(last_used);

            packages.push(Package {
                name,
                version,
                description,
                install_date: None,
                last_used,
                size_bytes,
                source: PackageSource::Flatpak,
                is_orphan: false,
                usage_tag,
                files: Vec::new(),
            });
        }

        Ok(packages)
    }

    async fn list_orphans(&self) -> Result<Vec<Package>, PmalError> {
        // List all installed runtimes
        let output = run_command(
            "flatpak",
            &[
                "list",
                "--runtime",
                "--columns=application,name,version,size,description",
            ],
        )
        .await?;
        let stdout = parse_stdout(&output)?;

        // Get list of runtimes actually used by apps
        let used_output = run_command(
            "flatpak",
            &["list", "--app", "--columns=runtime"],
        )
        .await;
        let used_runtimes: Vec<String> = if let Ok(uo) = used_output {
            if let Ok(us) = parse_stdout(&uo) {
                us.lines()
                    .map(|l| l.trim().to_string())
                    .filter(|l| !l.is_empty())
                    .collect()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        let mut all_runtimes: Vec<(String, String, String, u64, String)> = Vec::new();

        // Parse all runtimes
        for line in stdout.lines() {
            let fields: Vec<&str> = line.split('\t').collect();
            if fields.is_empty() {
                continue;
            }

            let app_id = fields[0].trim().to_string();
            let name = if fields.len() > 1 && !fields[1].trim().is_empty() {
                fields[1].trim().to_string()
            } else {
                app_id.clone()
            };
            let version = if fields.len() > 2 {
                fields[2].trim().to_string()
            } else {
                String::new()
            };
            let size_bytes = if fields.len() > 3 {
                Self::parse_flatpak_size(fields[3].trim())
            } else {
                0
            };
            let description = if fields.len() > 4 {
                fields[4].trim().to_string()
            } else {
                format!("Unused runtime: {}", app_id)
            };

            all_runtimes.push((app_id, name, version, size_bytes, description));
        }

        let mut orphans = Vec::new();

        for (app_id, name, version, size_bytes, description) in all_runtimes {
            // Skip if this runtime is explicitly used by an app
            if used_runtimes.iter().any(|r| r.contains(&app_id)) {
                continue;
            }

            // Skip known system extensions that are pulled in by runtimes, not apps
            if Self::is_system_extension(&app_id) {
                continue;
            }

            orphans.push(Package {
                name,
                version,
                description,
                install_date: None,
                last_used: None,
                size_bytes,
                source: PackageSource::Flatpak,
                is_orphan: true,
                usage_tag: crate::pmal::UsageTag::NeverLaunched,
                files: Vec::new(),
            });
        }

        Ok(orphans)
    }

    async fn get_reverse_deps(&self, pkg: &str) -> Result<Vec<String>, PmalError> {
        // For flatpak, check if any apps depend on this runtime
        let output = run_command(
            "flatpak",
            &["list", "--app", "--columns=application,runtime"],
        )
        .await?;
        let stdout = parse_stdout(&output)?;

        let deps: Vec<String> = stdout
            .lines()
            .filter(|l| l.contains(pkg))
            .filter_map(|l| l.split('\t').next())
            .map(|s| s.trim().to_string())
            .collect();

        Ok(deps)
    }

    async fn get_files(&self, pkg: &str) -> Result<Vec<String>, PmalError> {
        // Flatpak apps are in /var/lib/flatpak or ~/.local/share/flatpak
        let home = std::env::var("HOME").unwrap_or_default();
        let paths = vec![
            format!("/var/lib/flatpak/app/{}", pkg),
            format!("{}/.local/share/flatpak/app/{}", home, pkg),
        ];

        let mut files = Vec::new();
        for path in &paths {
            if std::path::Path::new(path).exists() {
                files.push(path.clone());
            }
        }

        Ok(files)
    }

    async fn remove(&self, pkg: &str, dry_run: bool) -> Result<RemovalResult, PmalError> {
        if dry_run {
            return Ok(RemovalResult {
                package_name: pkg.to_string(),
                success: true,
                message: format!("Dry run: would remove {} via flatpak uninstall", pkg),
                space_recovered_bytes: 0,
            });
        }

        let output = run_command(
            "flatpak",
            &["uninstall", "-y", pkg],
        )
        .await?;

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

impl FlatpakBackend {
    fn is_system_extension(app_id: &str) -> bool {
        // Known system extensions that are dependencies of runtimes, not orphans
        // These are typically:
        // - GPU drivers (.GL., .VAAPI.)
        // - Codecs (.Codecs.)
        // - Locale data (.Locale.)
        // - GL extensions
        // - Theme/shell extensions
        let extension_patterns = [
            ".GL.",
            ".VAAPI.",
            ".Codecs.",
            ".Locale.",
            ".Debug.",
            ".Docs.",
            ".Translations.",
            "org.freedesktop.Platform.GL",
            "org.freedesktop.Platform.VAAPI",
            "org.freedesktop.Platform.Codecs",
            "org.kde.Platform.GL",
            "org.gnome.Platform.GL",
        ];

        extension_patterns.iter().any(|pattern| app_id.contains(pattern))
    }

    fn parse_flatpak_size(s: &str) -> u64 {
        let s = s.trim();
        // Flatpak sizes can be like "123.4 MB" or "1.2 GB"
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
            "KB" | "KIB" | "K" => (num * 1024.0) as u64,
            "MB" | "MIB" | "M" => (num * 1024.0 * 1024.0) as u64,
            "GB" | "GIB" | "G" => (num * 1024.0 * 1024.0 * 1024.0) as u64,
            _ => num as u64,
        }
    }
}
