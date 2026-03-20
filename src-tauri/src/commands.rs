use crate::db::Database;
use crate::distro_detect;
use crate::pmal::{
    is_system_critical, Package, PackageManager, PackageSource,
    RemovalPreview, RemovalRecord, RemovalResult,
};
use crate::sysinfo_collector::{self, SystemInfo};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct AppState {
    pub managers: Vec<Box<dyn PackageManager>>,
    pub distro: distro_detect::DistroInfo,
    pub db: Database,
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

fn find_manager<'a>(
    managers: &'a [Box<dyn PackageManager>],
    source: &PackageSource,
) -> Option<&'a Box<dyn PackageManager>> {
    managers.iter().find(|m| &m.source() == source)
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

    all_packages.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
    info.top_packages_by_size = all_packages.iter()
        .take(5)
        .map(|p| (p.name.clone(), p.size_bytes))
        .collect();

    Ok(info)
}

#[tauri::command]
pub async fn get_all_packages(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<Vec<Package>, String> {
    let state = state.lock().await;
    let mut all_packages: Vec<Package> = Vec::new();

    for manager in &state.managers {
        match manager.list_user_installed().await {
            Ok(pkgs) => all_packages.extend(pkgs),
            Err(e) => {
                eprintln!("Error listing packages from {}: {}", manager.name(), e);
            }
        }
    }

    all_packages.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
    Ok(all_packages)
}

#[tauri::command]
pub async fn get_orphans(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<Vec<Package>, String> {
    let state = state.lock().await;
    let mut orphans: Vec<Package> = Vec::new();

    for manager in &state.managers {
        match manager.list_orphans().await {
            Ok(pkgs) => orphans.extend(pkgs),
            Err(e) => {
                eprintln!("Error listing orphans from {}: {}", manager.name(), e);
            }
        }
    }

    orphans.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
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
    manager.get_reverse_deps(&name).await.map_err(|e| e.to_string())
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
    let cmd = get_removal_command(&name, &source);

    // Estimate size from files
    let size: u64 = files.iter()
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

    let cmd = get_removal_command(&name, &source);
    let manager = find_manager(&state.managers, &source)
        .ok_or_else(|| format!("No manager found for source {:?}", source))?;

    let result = manager.remove(&name, false).await.map_err(|e| e.to_string())?;

    if result.success {
        state.db.record_removal(&name, &source, result.space_recovered_bytes, &cmd)
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
    state.db.get_removal_history().await.map_err(|e| e.to_string())
}
