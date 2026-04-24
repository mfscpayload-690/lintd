use crate::db::Database;
use crate::distro_detect;
use crate::pmal::{
    is_system_critical, parse_stdout, run_command, Package, PackageManager, PackageSource,
    RemovalPreview, RemovalRecord, RemovalResult,
};
use crate::sysinfo_collector::{self, SystemInfo};
use serde::Serialize;
use std::cmp::Reverse;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tauri::Emitter;
use tokio::sync::Mutex;

pub type SharedManager = Arc<Box<dyn PackageManager>>;

pub struct AppState {
    pub managers: Vec<SharedManager>,
    pub distro: distro_detect::DistroInfo,
    pub db: Database,
}

#[derive(Debug, Serialize)]
pub struct BackfillResult {
    pub scanned: u64,
    pub updated: u64,
    pub skipped: u64,
}

#[derive(Serialize, Clone)]
pub struct ScanProgressEvent {
    pub source: String,
    pub packages: Vec<Package>,
    pub done_count: usize,
    pub total_count: usize,
    pub error: Option<String>,
}

fn get_removal_command(name: &str, source: &PackageSource) -> String {
    match source {
        PackageSource::Pacman => format!("sudo pacman -Rns {}", name),
        PackageSource::Aur => format!("sudo pacman -Rns {}", name),
        PackageSource::Apt => format!("sudo apt-get remove --purge {}", name),
        PackageSource::Dnf => format!("sudo dnf remove {}", name),
        PackageSource::Apk => format!("sudo apk del {}", name),
        PackageSource::Nix => format!("nix-env --uninstall {}", name),
        PackageSource::Flatpak => format!("flatpak uninstall {}", name),
        PackageSource::Snap => format!("sudo snap remove {}", name),
        PackageSource::AppImage => format!("rm {}", name),
        PackageSource::Manual => format!("# manual removal of {}", name),
    }
}

async fn resolve_flatpak_ref(name: &str) -> String {
    if name.contains('.') {
        return name.to_string();
    }

    let app_output = run_command("flatpak", &["list", "--app", "--columns=application,name"]).await;

    if let Ok(output) = app_output {
        if let Ok(stdout) = parse_stdout(&output) {
            for line in stdout.lines() {
                let fields: Vec<&str> = line.split('\t').collect();
                if fields.len() >= 2 && fields[1].trim().eq_ignore_ascii_case(name) {
                    return fields[0].trim().to_string();
                }
            }
        }
    }

    let runtime_output = run_command(
        "flatpak",
        &["list", "--runtime", "--columns=application,name"],
    )
    .await;

    if let Ok(output) = runtime_output {
        if let Ok(stdout) = parse_stdout(&output) {
            for line in stdout.lines() {
                let fields: Vec<&str> = line.split('\t').collect();
                if fields.len() >= 2 && fields[1].trim().eq_ignore_ascii_case(name) {
                    return fields[0].trim().to_string();
                }
            }
        }
    }

    name.to_string()
}

fn extract_flatpak_ref_from_command(command: &str) -> Option<String> {
    let tokens: Vec<&str> = command.split_whitespace().collect();
    if tokens.len() < 3 || tokens.first().copied() != Some("flatpak") {
        return None;
    }

    let uninstall_index = tokens.iter().position(|t| *t == "uninstall")?;
    let ref_token = tokens
        .iter()
        .skip(uninstall_index + 1)
        .rev()
        .find(|t| !t.starts_with('-'))?;

    Some((*ref_token).to_string())
}

fn parse_flatpak_size_to_bytes(size: &str) -> u64 {
    let trimmed = size.trim();
    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    if parts.is_empty() {
        return 0;
    }

    let numeric = parts[0].replace(',', "");
    let num: f64 = numeric.parse().unwrap_or(0.0);
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

async fn estimate_flatpak_ref_size_bytes(pkg_ref: &str) -> u64 {
    let output = run_command("flatpak", &["info", "--show-size", pkg_ref]).await;
    let Ok(output) = output else {
        return 0;
    };

    let Ok(stdout) = parse_stdout(&output) else {
        return 0;
    };

    for line in stdout.lines() {
        let lower = line.to_lowercase();
        if !lower.contains("installed") {
            continue;
        }

        if let Some((_, value_part)) = line.split_once(':') {
            let value = value_part.trim();
            let human = value.split('(').next().unwrap_or(value).trim();
            let parsed = parse_flatpak_size_to_bytes(human);
            if parsed > 0 {
                return parsed;
            }
        }
    }

    0
}

fn find_manager<'a>(
    managers: &'a [SharedManager],
    source: &PackageSource,
) -> Option<&'a SharedManager> {
    managers.iter().find(|m| &m.source() == source)
}

pub async fn collect_packages_concurrent(managers: Vec<SharedManager>) -> Vec<Package> {
    let tasks: Vec<_> = managers
        .into_iter()
        .map(|manager| {
            tokio::task::spawn(async move {
                match manager.list_user_installed().await {
                    Ok(pkgs) => pkgs,
                    Err(e) => {
                        eprintln!("Error listing packages from {}: {}", manager.name(), e);
                        vec![]
                    }
                }
            })
        })
        .collect();

    let results = futures::future::join_all(tasks).await;

    let mut packages: Vec<Package> = results
        .into_iter()
        .filter_map(|r| r.ok())
        .flatten()
        .collect();

    packages.sort_by_key(|p| Reverse(p.size_bytes));
    packages
}

pub async fn collect_orphans_concurrent(managers: Vec<SharedManager>) -> Vec<Package> {
    let tasks: Vec<_> = managers
        .into_iter()
        .map(|manager| {
            tokio::task::spawn(async move {
                match manager.list_orphans().await {
                    Ok(pkgs) => pkgs,
                    Err(e) => {
                        eprintln!("Error listing orphans from {}: {}", manager.name(), e);
                        vec![]
                    }
                }
            })
        })
        .collect();

    let results = futures::future::join_all(tasks).await;

    let mut packages: Vec<Package> = results
        .into_iter()
        .filter_map(|r| r.ok())
        .flatten()
        .collect();

    packages.sort_by_key(|p| Reverse(p.size_bytes));
    packages
}

#[tauri::command]
pub async fn get_system_info(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<SystemInfo, String> {
    let state = state.lock().await;
    let mut info = sysinfo_collector::collect_system_info(&state.distro);

    // Collect top packages by size
    let mut all_packages: Vec<Package> = Vec::new();
    for manager in &state.managers {
        match manager.list_user_installed().await {
            Ok(pkgs) => all_packages.extend(pkgs),
            Err(_) => continue,
        }
    }

    all_packages.sort_by_key(|p| Reverse(p.size_bytes));
    info.top_packages_by_size = all_packages
        .iter()
        .take(5)
        .map(|p| (p.name.clone(), p.size_bytes))
        .collect();

    info.package_managers = state
        .managers
        .iter()
        .map(|m| m.name().to_string())
        .collect();

    Ok(info)
}

#[tauri::command]
pub async fn get_all_packages(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<Vec<Package>, String> {
    let managers: Vec<SharedManager> = {
        let state = state.lock().await;
        state.managers.iter().map(Arc::clone).collect()
    };
    let packages = collect_packages_concurrent(managers).await;
    Ok(packages)
}

#[tauri::command]
pub async fn get_orphans(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<Vec<Package>, String> {
    let managers: Vec<SharedManager> = {
        let state = state.lock().await;
        state.managers.iter().map(Arc::clone).collect()
    };
    let orphans = collect_orphans_concurrent(managers).await;
    Ok(orphans)
}

#[tauri::command]
pub async fn get_package_files(
    name: String,
    source: PackageSource,
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<Vec<String>, String> {
    let state = state.lock().await;
    let manager = find_manager(&state.managers, &source)
        .ok_or_else(|| format!("No manager found for source {:?}", source))?;
    manager.get_files(&name).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_reverse_deps(
    name: String,
    source: PackageSource,
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<Vec<String>, String> {
    let state = state.lock().await;
    let manager = find_manager(&state.managers, &source)
        .ok_or_else(|| format!("No manager found for source {:?}", source))?;
    manager
        .get_reverse_deps(&name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn preview_removal(
    name: String,
    source: PackageSource,
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<RemovalPreview, String> {
    let state = state.lock().await;
    let manager = find_manager(&state.managers, &source)
        .ok_or_else(|| format!("No manager found for source {:?}", source))?;

    let files = manager.get_files(&name).await.unwrap_or_default();
    let reverse_deps = manager.get_reverse_deps(&name).await.unwrap_or_default();
    let critical = is_system_critical(&name);
    let safe = reverse_deps.is_empty() && !critical;
    let cmd_target = if source == PackageSource::Flatpak {
        resolve_flatpak_ref(&name).await
    } else {
        name.clone()
    };
    let cmd = get_removal_command(&cmd_target, &source);

    // Estimate size from files
    let size: u64 = files
        .iter()
        .filter_map(|f| std::fs::metadata(f).ok())
        .map(|m| m.len())
        .sum();

    Ok(RemovalPreview {
        package_name: name.clone(),
        description: String::new(),
        files_to_delete: files,
        reverse_deps,
        is_system_critical: critical,
        size_to_recover_bytes: size,
        cli_command_preview: cmd,
        safe_to_remove: safe,
    })
}

#[tauri::command]
pub async fn execute_removal(
    name: String,
    source: PackageSource,
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<RemovalResult, String> {
    let state = state.lock().await;

    if is_system_critical(&name) {
        return Err("Cannot remove system-critical package".into());
    }

    let cmd_target = if source == PackageSource::Flatpak {
        resolve_flatpak_ref(&name).await
    } else {
        name.clone()
    };
    let cmd = get_removal_command(&cmd_target, &source);
    let manager = find_manager(&state.managers, &source)
        .ok_or_else(|| format!("No manager found for source {:?}", source))?;

    let result = manager
        .remove(&name, false)
        .await
        .map_err(|e| e.to_string())?;

    if result.success {
        state
            .db
            .record_removal(&name, &source, result.space_recovered_bytes, &cmd)
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(result)
}

#[tauri::command]
pub async fn get_removal_history(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<Vec<RemovalRecord>, String> {
    let state = state.lock().await;
    state
        .db
        .get_removal_history()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn backfill_flatpak_history_sizes(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<BackfillResult, String> {
    let state = state.lock().await;
    let candidates = state
        .db
        .get_flatpak_zero_space_history()
        .await
        .map_err(|e| e.to_string())?;

    let mut updated = 0u64;
    let mut skipped = 0u64;

    for (id, command) in &candidates {
        let Some(pkg_ref) = extract_flatpak_ref_from_command(command) else {
            skipped += 1;
            continue;
        };

        let size = estimate_flatpak_ref_size_bytes(&pkg_ref).await;
        if size == 0 {
            skipped += 1;
            continue;
        }

        state
            .db
            .update_removal_space_recovered(*id, size)
            .await
            .map_err(|e| e.to_string())?;
        updated += 1;
    }

    Ok(BackfillResult {
        scanned: candidates.len() as u64,
        updated,
        skipped,
    })
}

#[tauri::command]
pub async fn scan_packages_streaming(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<(), String> {
    let managers: Vec<SharedManager> = {
        let state = state.lock().await;
        state.managers.iter().map(Arc::clone).collect()
    };

    let total_count = managers.len();
    let done_count = Arc::new(AtomicUsize::new(0));

    let tasks: Vec<_> = managers
        .into_iter()
        .map(|manager| {
            let app = app.clone();
            let done_count = Arc::clone(&done_count);
            tokio::task::spawn(async move {
                let result = manager.list_user_installed().await;
                let done = done_count.fetch_add(1, Ordering::SeqCst) + 1;
                let (packages, error) = match result {
                    Ok(pkgs) => (pkgs, None),
                    Err(e) => (vec![], Some(e.to_string())),
                };
                let event = ScanProgressEvent {
                    source: manager.name().to_string(),
                    packages,
                    done_count: done,
                    total_count,
                    error,
                };
                let _ = app.emit("scan_progress", event);
            })
        })
        .collect();

    futures::future::join_all(tasks).await;
    Ok(())
}
