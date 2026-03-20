use crate::pmal::{
    compute_usage_tag, get_desktop_atime, run_command, parse_stdout,
    Package, PackageManager, PackageSource, PmalError, RemovalResult,
};
use chrono::{DateTime, NaiveDateTime, Utc};

pub struct AptBackend;

impl AptBackend {
    pub fn new() -> Self {
        Self
    }

    fn parse_dpkg_list(stdout: &str) -> Vec<Package> {
        let mut packages = Vec::new();

        for line in stdout.lines() {
            // dpkg-query format: name\tversion\tsize\tdescription\tinstall_date_epoch
            let fields: Vec<&str> = line.split('\t').collect();
            if fields.len() < 4 {
                continue;
            }

            let name = fields[0].trim().to_string();
            let version = fields[1].trim().to_string();
            let size_bytes: u64 = fields[2].trim().parse().unwrap_or(0) * 1024; // dpkg reports in KB
            let description = fields[3].trim().to_string();

            let last_used = get_desktop_atime(&name);
            let usage_tag = compute_usage_tag(last_used);

            // Try to get install date from dpkg info dir
            let install_date = Self::get_install_date(&name);

            packages.push(Package {
                name,
                version,
                description,
                install_date,
                last_used,
                size_bytes,
                source: PackageSource::Apt,
                is_orphan: false,
                usage_tag,
                files: Vec::new(),
            });
        }

        packages
    }

    fn get_install_date(pkg_name: &str) -> Option<DateTime<Utc>> {
        let list_path = format!("/var/lib/dpkg/info/{}.list", pkg_name);
        if let Ok(metadata) = std::fs::metadata(&list_path) {
            if let Ok(modified) = metadata.modified() {
                let dt: DateTime<Utc> = modified.into();
                return Some(dt);
            }
        }
        None
    }
}

#[async_trait::async_trait]
impl PackageManager for AptBackend {
    fn name(&self) -> &str {
        "apt"
    }

    fn source(&self) -> PackageSource {
        PackageSource::Apt
    }

    fn detect(&self) -> bool {
        std::path::Path::new("/usr/bin/apt").exists()
            || std::path::Path::new("/usr/bin/dpkg").exists()
    }

    async fn list_user_installed(&self) -> Result<Vec<Package>, PmalError> {
        // Get manually installed packages using apt-mark
        let output = run_command(
            "dpkg-query",
            &[
                "-W",
                "-f=${Package}\\t${Version}\\t${Installed-Size}\\t${Description}\\n",
            ],
        )
        .await?;
        let stdout = parse_stdout(&output)?;

        // Get list of manually installed
        let manual_output = run_command("apt-mark", &["showmanual"]).await;
        let manual_names: Vec<String> = if let Ok(mo) = manual_output {
            if let Ok(ms) = parse_stdout(&mo) {
                ms.lines().map(|l| l.trim().to_string()).collect()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        let mut packages = Self::parse_dpkg_list(&stdout);

        // Filter to only manually installed if we got the list
        if !manual_names.is_empty() {
            packages.retain(|p| manual_names.contains(&p.name));
        }

        Ok(packages)
    }

    async fn list_orphans(&self) -> Result<Vec<Package>, PmalError> {
        // Try deborphan first, fall back to apt autoremove simulation
        let output = run_command("deborphan", &[]).await;
        let orphan_names: Vec<String> = if let Ok(o) = output {
            if let Ok(s) = parse_stdout(&o) {
                s.lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect()
            } else {
                Vec::new()
            }
        } else {
            // Fall back to apt autoremove --dry-run
            let ar_output = run_command("apt-get", &["autoremove", "--dry-run"]).await?;
            let ar_stdout = parse_stdout(&ar_output)?;
            ar_stdout
                .lines()
                .filter(|l| l.starts_with("Remv "))
                .filter_map(|l| l.split_whitespace().nth(1))
                .map(String::from)
                .collect()
        };

        if orphan_names.is_empty() {
            return Ok(Vec::new());
        }

        let names_str = orphan_names.join(",");
        let output = run_command(
            "dpkg-query",
            &[
                "-W",
                "-f=${Package}\\t${Version}\\t${Installed-Size}\\t${Description}\\n",
                &names_str,
            ],
        )
        .await;

        let mut packages = if let Ok(o) = output {
            if let Ok(s) = parse_stdout(&o) {
                Self::parse_dpkg_list(&s)
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        for pkg in &mut packages {
            pkg.is_orphan = true;
        }

        Ok(packages)
    }

    async fn get_reverse_deps(&self, pkg: &str) -> Result<Vec<String>, PmalError> {
        let output = run_command("apt-cache", &["rdepends", "--installed", pkg]).await?;
        let stdout = parse_stdout(&output)?;

        let deps: Vec<String> = stdout
            .lines()
            .skip(2) // Skip header lines
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty() && !l.starts_with('|'))
            .collect();

        Ok(deps)
    }

    async fn get_files(&self, pkg: &str) -> Result<Vec<String>, PmalError> {
        let output = run_command("dpkg", &["-L", pkg]).await?;
        let stdout = parse_stdout(&output)?;

        let files: Vec<String> = stdout
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
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

        let output =
            run_command("pkexec", &["apt-get", "remove", "--purge", "-y", pkg]).await?;

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
