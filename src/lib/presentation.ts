import type { PackageSource, UsageTag } from "../types/lintd";

export const PACKAGE_SOURCES: PackageSource[] = [
  "pacman",
  "aur",
  "apt",
  "dnf",
  "flatpak",
  "snap",
  "appimage",
  "apk",
  "nix",
  "manual",
];

export const USAGE_TAGS: UsageTag[] = ["active", "rarely_used", "never_launched"];

export const sourceLabelMap: Record<PackageSource, string> = {
  pacman: "Pacman",
  aur: "AUR",
  apt: "APT",
  dnf: "DNF",
  flatpak: "Flatpak",
  snap: "Snap",
  appimage: "AppImage",
  apk: "APK",
  nix: "Nix",
  manual: "Manual",
};

export const sourceBadgeClassMap: Record<PackageSource, string> = {
  pacman: "bg-slate-100 text-slate-800 dark:bg-slate-800 dark:text-slate-100",
  aur: "bg-amber-100 text-amber-800 dark:bg-amber-950 dark:text-amber-300",
  apt: "bg-blue-100 text-blue-800 dark:bg-blue-950 dark:text-blue-300",
  dnf: "bg-red-100 text-red-800 dark:bg-red-950 dark:text-red-300",
  flatpak: "bg-indigo-100 text-indigo-800 dark:bg-indigo-950 dark:text-indigo-300",
  snap: "bg-emerald-100 text-emerald-800 dark:bg-emerald-950 dark:text-emerald-300",
  appimage: "bg-violet-100 text-violet-800 dark:bg-violet-950 dark:text-violet-300",
  apk: "bg-cyan-100 text-cyan-800 dark:bg-cyan-950 dark:text-cyan-300",
  nix: "bg-teal-100 text-teal-800 dark:bg-teal-950 dark:text-teal-300",
  manual: "bg-zinc-100 text-zinc-800 dark:bg-zinc-800 dark:text-zinc-200",
};

export const usageLabelMap: Record<UsageTag, string> = {
  active: "Active",
  rarely_used: "Rarely Used",
  never_launched: "Never Launched",
};

export const usageBadgeClassMap: Record<UsageTag, string> = {
  active: "bg-emerald-100 text-emerald-800 dark:bg-emerald-950 dark:text-emerald-300",
  rarely_used: "bg-amber-100 text-amber-800 dark:bg-amber-950 dark:text-amber-300",
  never_launched: "bg-red-100 text-red-800 dark:bg-red-950 dark:text-red-300",
};
