use crate::pmal::{
    compute_usage_tag, get_last_used_time, run_command, parse_stdout,
    Package, PackageManager, PackageSource, PmalError, RemovalResult,
};
use chrono::DateTime;

pub struct DnfBackend;

impl DnfBackend {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl PackageManager for DnfBackend {
    fn name(&self) -> &str {
        "dnf"
    }

    fn source(&self) -> PackageSource {
        PackageSource::Dnf
    }

    fn detect(&self) -> bool {
        std::path::Path::new("/usr/bin/dnf").exists()
    }

    async fn list_user_installed(&self) -> Result<Vec<Package>, PmalError> {
        let output = run_command(
            "dnf",
            &["repoquery", "--installed", "--userinstalled", "--qf",
              "%{name}\\t%{version}-%{release}\\t%{installsize}\\t%{summary}\\t%{installtime}"],
        )
        .await?;
        let stdout = parse_stdout(&output)?;

        let mut packages = Vec::new();
        for line in stdout.lines() {
            let fields: Vec<&str> = line.split('\t').collect();
            if fields.len() < 5 {
                continue;
            }

            let name = fields[0].trim().to_string();
            let version = fields[1].trim().to_string();
            let size_bytes: u64 = fields[2].trim().parse().unwrap_or(0);
            let description = fields[3].trim().to_string();
            let install_epoch: i64 = fields[4].trim().parse().unwrap_or(0);

            let install_date = if install_epoch > 0 {
                DateTime::from_timestamp(install_epoch, 0)
            } else {
                None
            };

            let last_used = get_last_used_time(&name, &[]);
            let usage_tag = compute_usage_tag(last_used);

            packages.push(Package {
                name,
                version,
                description,
                install_date,
                last_used,
                size_bytes,
                source: PackageSource::Dnf,
                is_orphan: false,
                usage_tag,
                files: Vec::new(),
            });
        }

        Ok(packages)
    }

    async fn list_orphans(&self) -> Result<Vec<Package>, PmalError> {
        let output = run_command(
            "dnf",
            &["repoquery", "--extras", "--qf",
              "%{name}\\t%{version}-%{release}\\t%{installsize}\\t%{summary}\\t%{installtime}"],
        )
        .await?;
        let stdout = parse_stdout(&output)?;

        let mut packages = Vec::new();
        for line in stdout.lines() {
            let fields: Vec<&str> = line.split('\t').collect();
            if fields.len() < 5 {
                continue;
            }

            let name = fields[0].trim().to_string();
            let version = fields[1].trim().to_string();
            let size_bytes: u64 = fields[2].trim().parse().unwrap_or(0);
            let description = fields[3].trim().to_string();
            let install_epoch: i64 = fields[4].trim().parse().unwrap_or(0);

            let install_date = if install_epoch > 0 {
                DateTime::from_timestamp(install_epoch, 0)
            } else {
                None
            };

            let last_used = get_last_used_time(&name, &[]);
            let usage_tag = compute_usage_tag(last_used);

            packages.push(Package {
                name,
                version,
                description,
                install_date,
                last_used,
                size_bytes,
                source: PackageSource::Dnf,
                is_orphan: true,
                usage_tag,
                files: Vec::new(),
            });
        }

        Ok(packages)
    }

    async fn get_reverse_deps(&self, pkg: &str) -> Result<Vec<String>, PmalError> {
        let output = run_command(
            "dnf",
            &["repoquery", "--installed", "--whatrequires", pkg, "--qf", "%{name}"],
        )
        .await?;
        let stdout = parse_stdout(&output)?;

        let deps: Vec<String> = stdout
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect();

        Ok(deps)
    }

    async fn get_files(&self, pkg: &str) -> Result<Vec<String>, PmalError> {
        let output = run_command("rpm", &["-ql", pkg]).await?;
        let stdout = parse_stdout(&output)?;

        let files: Vec<String> = stdout
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty() && !l.starts_with("(contains"))
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

        let output = run_command("pkexec", &["dnf", "remove", "-y", pkg]).await?;

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
