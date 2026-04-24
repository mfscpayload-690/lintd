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
  pacman: "bg-muted text-muted-foreground font-mono text-xs",
  aur: "bg-muted text-muted-foreground font-mono text-xs",
  apt: "bg-muted text-muted-foreground font-mono text-xs",
  dnf: "bg-muted text-muted-foreground font-mono text-xs",
  flatpak: "bg-muted text-muted-foreground font-mono text-xs",
  snap: "bg-muted text-muted-foreground font-mono text-xs",
  appimage: "bg-muted text-muted-foreground font-mono text-xs",
  apk: "bg-muted text-muted-foreground font-mono text-xs",
  nix: "bg-muted text-muted-foreground font-mono text-xs",
  manual: "bg-muted text-muted-foreground font-mono text-xs",
};

export const usageLabelMap: Record<UsageTag, string> = {
  active: "Active",
  rarely_used: "Rarely Used",
  never_launched: "Never Launched",
};

export const usageBadgeClassMap: Record<UsageTag, string> = {
  active: "text-foreground font-mono text-xs",
  rarely_used: "text-amber-600 dark:text-amber-400 font-mono text-xs",
  never_launched: "text-destructive font-mono text-xs",
};
