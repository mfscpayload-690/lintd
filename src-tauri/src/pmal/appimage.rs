use crate::pmal::{
    compute_usage_tag, Package, PackageManager, PackageSource, PmalError, RemovalResult,
};
use chrono::{DateTime, Utc};

pub struct AppImageBackend;

impl AppImageBackend {
    pub fn new() -> Self {
        Self
    }

    fn scan_dirs() -> Vec<String> {
        let home = std::env::var("HOME").unwrap_or_default();
        vec![
            format!("{}/Applications", home),
            format!("{}/.local/bin", home),
            format!("{}/Desktop", home),
            "/opt".to_string(),
        ]
    }

    fn find_appimages() -> Vec<(String, std::fs::Metadata)> {
        let mut results = Vec::new();
        for dir in Self::scan_dirs() {
            let path = std::path::Path::new(&dir);
            if !path.exists() {
                continue;
            }
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if let Some(ext) = p.extension() {
                        if ext.to_string_lossy().to_lowercase() == "appimage" {
                            if let Ok(meta) = entry.metadata() {
                                results.push((p.to_string_lossy().to_string(), meta));
                            }
                        }
                    }
                }
            }
        }
        results
    }
}

#[async_trait::async_trait]
impl PackageManager for AppImageBackend {
    fn name(&self) -> &str {
        "appimage"
    }
    fn source(&self) -> PackageSource {
        PackageSource::AppImage
    }
    fn detect(&self) -> bool {
        true
    } // Always available as a FS scanner

    async fn list_user_installed(&self) -> Result<Vec<Package>, PmalError> {
        let appimages = Self::find_appimages();
        let mut packages = Vec::new();

        for (path, meta) in appimages {
            let filename = std::path::Path::new(&path)
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| path.clone());

            let size_bytes = meta.len();
            let last_used = meta.accessed().ok().map(|t| {
                let dt: DateTime<Utc> = t.into();
                dt
            });
            let install_date = meta.modified().ok().map(|t| {
                let dt: DateTime<Utc> = t.into();
                dt
            });
            let usage_tag = compute_usage_tag(last_used);

            packages.push(Package {
                name: filename,
                version: "N/A".to_string(),
                description: format!("AppImage at {}", path),
                install_date,
                last_used,
                size_bytes,
                source: PackageSource::AppImage,
                is_orphan: false,
                usage_tag,
                files: vec![path],
            });
        }
        Ok(packages)
    }

    async fn list_orphans(&self) -> Result<Vec<Package>, PmalError> {
        Ok(Vec::new()) // AppImages don't have orphan concept
    }

    async fn get_reverse_deps(&self, _pkg: &str) -> Result<Vec<String>, PmalError> {
        Ok(Vec::new()) // Self-contained
    }

    async fn get_files(&self, pkg: &str) -> Result<Vec<String>, PmalError> {
        let appimages = Self::find_appimages();
        let files: Vec<String> = appimages
            .into_iter()
            .filter(|(p, _)| p.contains(pkg))
            .map(|(p, _)| p)
            .collect();
        Ok(files)
    }

    async fn remove(&self, pkg: &str, dry_run: bool) -> Result<RemovalResult, PmalError> {
        let appimages = Self::find_appimages();
        let target = appimages.iter().find(|(p, _)| p.contains(pkg));

        match target {
            Some((path, meta)) => {
                let size = meta.len();
                if dry_run {
                    return Ok(RemovalResult {
                        package_name: pkg.into(),
                        success: true,
                        message: format!("Dry run: would delete {}", path),
                        space_recovered_bytes: size,
                    });
                }
                std::fs::remove_file(path).map_err(PmalError::IoError)?;
                Ok(RemovalResult {
                    package_name: pkg.into(),
                    success: true,
                    message: format!("Deleted {}", path),
                    space_recovered_bytes: size,
                })
            }
            None => Err(PmalError::PackageNotFound(pkg.into())),
        }
    }
}
