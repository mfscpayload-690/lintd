export type DistroFamily =
  | 'arch_family'
  | 'debian_family'
  | 'fedora_family'
  | 'alpine_family'
  | 'nix_family'
  | 'suse_family'
  | 'unknown';

export interface DistroInfo {
  id: string;
  name: string;
  version: string;
  family: DistroFamily;
  logo_name: string;
}

export interface MountPoint {
  path: string;
  total_bytes: number;
  used_bytes: number;
  free_bytes: number;
  fs_type: string;
}

export interface SystemInfo {
  distro_id: string;
  distro_id_like: string;
  distro_name: string;
  distro_version: string;
  distro_logo_name: string;
  kernel_version: string;
  hostname: string;
  username: string;
  de_wm: string;
  shell: string;
  cpu_model: string;
  cpu_cores: number;
  ram_total_mb: number;
  ram_used_mb: number;
  uptime_seconds: number;
  storage: MountPoint[];
  top_packages_by_size: [string, number][]; // Tuple of [name, size_bytes]
}

export type PackageSource =
  | 'pacman'
  | 'aur'
  | 'apt'
  | 'dnf'
  | 'flatpak'
  | 'snap'
  | 'appimage'
  | 'apk'
  | 'nix'
  | 'manual';

export type UsageTag = 'active' | 'rarely_used' | 'never_launched';

export interface Package {
  name: string;
  version: string;
  description: string;
  install_date: string | null; // ISO 8601 string from chrono
  last_used: string | null; // ISO 8601 string from chrono
  size_bytes: number;
  source: PackageSource;
  is_orphan: boolean;
  usage_tag: UsageTag;
  // files is marked #[serde(skip_serializing)] in Rust, so it won't be in the payload
  // We use get_package_files() to fetch them on demand.
}

export interface RemovalPreview {
  package_name: string;
  description: string;
  files_to_delete: string[];
  reverse_deps: string[];
  is_system_critical: boolean;
  size_to_recover_bytes: number;
  cli_command_preview: string;
  safe_to_remove: boolean;
}

export interface RemovalResult {
  package_name: string;
  success: boolean;
  message: string;
  space_recovered_bytes: number;
}

export interface RemovalRecord {
  id: number;
  package_name: string;
  source: PackageSource;
  removed_at: string; // ISO 8601 string from chrono
  space_recovered_bytes: number;
  command_executed: string;
}
