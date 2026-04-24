use serde::{Deserialize, Serialize};
use std::process::Command;
use sysinfo::{Disks, System};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountPoint {
    pub path: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub free_bytes: u64,
    pub fs_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub distro_id: String,
    pub distro_id_like: String,
    pub distro_name: String,
    pub distro_version: String,
    pub distro_logo_name: String,
    pub kernel_version: String,
    pub hostname: String,
    pub username: String,
    pub de_wm: String,
    pub shell: String,
    pub cpu_model: String,
    pub cpu_cores: u32,
    pub cpu_usage_percent: f32,
    pub ram_total_mb: u64,
    pub ram_used_mb: u64,
    pub gpu_name: Option<String>,
    pub gpu_vram_used_mb: Option<u64>,
    pub gpu_vram_total_mb: Option<u64>,
    pub uptime_seconds: u64,
    pub storage: Vec<MountPoint>,
    pub top_packages_by_size: Vec<(String, u64)>,
    pub package_managers: Vec<String>,
}

/// Detect NVIDIA GPU using nvidia-smi
fn detect_nvidia_gpu() -> Option<(String, u64, u64)> {
    let output = Command::new("nvidia-smi")
        .args([
            "--query-gpu=name,memory.used,memory.total",
            "--format=csv,noheader,nounits",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let line = String::from_utf8(output.stdout).ok()?;
    let parts: Vec<&str> = line.trim().split(',').collect();

    if parts.len() < 3 {
        return None;
    }

    let name = parts[0].trim().to_string();
    let used_mb = parts[1].trim().parse::<u64>().ok()?;
    let total_mb = parts[2].trim().parse::<u64>().ok()?;

    Some((name, used_mb, total_mb))
}

pub fn collect_system_info(distro: &crate::distro_detect::DistroInfo) -> SystemInfo {
    let mut sys = System::new_all();
    sys.refresh_all();
    std::thread::sleep(std::time::Duration::from_millis(300));
    sys.refresh_cpu_all();

    let kernel_version = System::kernel_version().unwrap_or_else(|| "Unknown".into());
    let hostname = System::host_name().unwrap_or_else(|| "Unknown".into());
    let username = std::env::var("USER")
        .or_else(|_| std::env::var("LOGNAME"))
        .unwrap_or_else(|_| "Unknown".into());

    let de_wm = std::env::var("XDG_CURRENT_DESKTOP")
        .or_else(|_| std::env::var("DESKTOP_SESSION"))
        .unwrap_or_else(|_| "Unknown".into());

    let shell = std::env::var("SHELL").unwrap_or_else(|_| "Unknown".into());

    let cpu_model = sys
        .cpus()
        .first()
        .map(|c| c.brand().to_string())
        .unwrap_or_else(|| "Unknown".into());
    let cpu_cores = sys.cpus().len() as u32;
    let cpu_usage_percent = if sys.cpus().is_empty() {
        0.0
    } else {
        (sys.cpus().iter().map(|c| c.cpu_usage()).sum::<f32>() / sys.cpus().len() as f32).min(100.0)
    };

    let ram_total_mb = sys.total_memory() / (1024 * 1024);
    let ram_used_mb = sys.used_memory() / (1024 * 1024);
    let uptime_seconds = System::uptime();

    // Detect NVIDIA GPU
    let (gpu_name, gpu_vram_used_mb, gpu_vram_total_mb) = detect_nvidia_gpu()
        .map(|(name, used, total)| (Some(name), Some(used), Some(total)))
        .unwrap_or((None, None, None));

    let disks = Disks::new_with_refreshed_list();
    let storage: Vec<MountPoint> = disks
        .list()
        .iter()
        .filter(|d| {
            let mount = d.mount_point().to_string_lossy().to_string();
            !mount.starts_with("/snap/")
                && !mount.starts_with("/sys")
                && !mount.starts_with("/proc")
                && !mount.starts_with("/dev")
                && !mount.starts_with("/run")
                && mount != "/boot/efi"
        })
        .map(|d| {
            let total = d.total_space();
            let free = d.available_space();
            MountPoint {
                path: d.mount_point().to_string_lossy().to_string(),
                total_bytes: total,
                used_bytes: total.saturating_sub(free),
                free_bytes: free,
                fs_type: d.file_system().to_string_lossy().to_string(),
            }
        })
        .collect();

    SystemInfo {
        distro_id: distro.id.clone(),
        distro_id_like: distro.id_like.clone(),
        distro_name: distro.name.clone(),
        distro_version: distro.version.clone(),
        distro_logo_name: distro.logo_name.clone(),
        kernel_version,
        hostname,
        username,
        de_wm,
        shell,
        cpu_model,
        cpu_cores,
        cpu_usage_percent,
        ram_total_mb,
        ram_used_mb,
        gpu_name,
        gpu_vram_used_mb,
        gpu_vram_total_mb,
        uptime_seconds,
        storage,
        top_packages_by_size: Vec::new(), // Populated after package scan
        package_managers: Vec::new(),     // Populated in get_system_info command
    }
}
