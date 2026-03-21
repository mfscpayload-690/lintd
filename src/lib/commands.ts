import { invoke } from '@tauri-apps/api/core';
import type {
  BackfillResult,
  SystemInfo,
  Package,
  PackageSource,
  RemovalPreview,
  RemovalResult,
  RemovalRecord,
} from '../types/lintd';

/**
 * Gets the current system overview, hardware stats, mount points, and distro info.
 */
export async function getSystemInfo(): Promise<SystemInfo> {
  return invoke<SystemInfo>('get_system_info');
}

/**
 * Lists all user-installed packages across all detected package managers.
 */
export async function getAllPackages(): Promise<Package[]> {
  return invoke<Package[]>('get_all_packages');
}

/**
 * Lists packages identified as orphans or unused dependencies.
 */
export async function getOrphans(): Promise<Package[]> {
  return invoke<Package[]>('get_orphans');
}

/**
 * Given a package name and its source, fetch the list of files installed by it.
 */
export async function getPackageFiles(
  name: string,
  source: PackageSource
): Promise<string[]> {
  return invoke<string[]>('get_package_files', { name, source });
}

/**
 * Fetch a list of packages that depend on this one. If not empty, it's unsafe to remove.
 */
export async function getReverseDeps(
  name: string,
  source: PackageSource
): Promise<string[]> {
  return invoke<string[]>('get_reverse_deps', { name, source });
}

/**
 * Generates a preview payload showing what will happen if this package is removed.
 * Populates files, reverse deps, critical package flags, and the actual CLI command.
 */
export async function previewRemoval(
  name: string,
  source: PackageSource
): Promise<RemovalPreview> {
  return invoke<RemovalPreview>('preview_removal', { name, source });
}

/**
 * Executes a privilege-escalated command via pkexec/polkit to permanently remove a package.
 */
export async function executeRemoval(
  name: string,
  source: PackageSource
): Promise<RemovalResult> {
  return invoke<RemovalResult>('execute_removal', { name, source });
}

/**
 * Fetches the historically persisted history of package removals from the SQLite layer.
 */
export async function getRemovalHistory(): Promise<RemovalRecord[]> {
  return invoke<RemovalRecord[]>('get_removal_history');
}

/**
 * Recalculates historical Flatpak removal rows that still have 0 B recovered.
 */
export async function backfillFlatpakHistorySizes(): Promise<BackfillResult> {
  return invoke<BackfillResult>('backfill_flatpak_history_sizes');
}
