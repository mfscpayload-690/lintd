use serde::{Deserialize, Serialize};
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
    pub ram_total_mb: u64,
    pub ram_used_mb: u64,
    pub uptime_seconds: u64,
    pub storage: Vec<MountPoint>,
    pub top_packages_by_size: Vec<(String, u64)>,
}

pub fn collect_system_info(
    distro: &crate::distro_detect::DistroInfo,
) -> SystemInfo {
    let mut sys = System::new_all();
    sys.refresh_all();

    let kernel_version = System::kernel_version().unwrap_or_else(|| "Unknown".into());
    let hostname = System::host_name().unwrap_or_else(|| "Unknown".into());
    let username = std::env::var("USER")
        .or_else(|_| std::env::var("LOGNAME"))
        .unwrap_or_else(|_| "Unknown".into());

    let de_wm = std::env::var("XDG_CURRENT_DESKTOP")
        .or_else(|_| std::env::var("DESKTOP_SESSION"))
        .unwrap_or_else(|_| "Unknown".into());

    let shell = std::env::var("SHELL").unwrap_or_else(|_| "Unknown".into());

    let cpu_model = sys.cpus().first()
        .map(|c| c.brand().to_string())
        .unwrap_or_else(|| "Unknown".into());
    let cpu_cores = sys.cpus().len() as u32;

    let ram_total_mb = sys.total_memory() / (1024 * 1024);
    let ram_used_mb = sys.used_memory() / (1024 * 1024);
    let uptime_seconds = System::uptime();

    let disks = Disks::new_with_refreshed_list();
    let storage: Vec<MountPoint> = disks.list().iter()
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
        ram_total_mb,
        ram_used_mb,
        uptime_seconds,
        storage,
        top_packages_by_size: Vec::new(), // Populated after package scan
    }
}
