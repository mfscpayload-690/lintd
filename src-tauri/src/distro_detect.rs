use crate::pmal::{self, PackageManager};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DistroFamily {
    ArchFamily,
    DebianFamily,
    FedoraFamily,
    AlpineFamily,
    NixFamily,
    SuseFamily,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistroInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub family: DistroFamily,
    pub logo_name: String,
}

pub fn detect_distro() -> DistroInfo {
    let os_release = std::fs::read_to_string("/etc/os-release").unwrap_or_default();
    let mut fields: HashMap<String, String> = HashMap::new();

    for line in os_release.lines() {
        if let Some((key, value)) = line.split_once('=') {
            let value = value.trim_matches('"').to_string();
            fields.insert(key.to_string(), value);
        }
    }

    let id = fields.get("ID").cloned().unwrap_or_else(|| "unknown".into());
    let name = fields.get("PRETTY_NAME")
        .or_else(|| fields.get("NAME"))
        .cloned()
        .unwrap_or_else(|| "Unknown Linux".into());
    let version = fields.get("VERSION_ID")
        .or_else(|| fields.get("VERSION"))
        .cloned()
        .unwrap_or_default();
    let id_like = fields.get("ID_LIKE").cloned().unwrap_or_default();

    let (family, logo_name) = match id.as_str() {
        "arch" => (DistroFamily::ArchFamily, "arch"),
        "manjaro" => (DistroFamily::ArchFamily, "manjaro"),
        "garuda" => (DistroFamily::ArchFamily, "garuda"),
        "endeavouros" => (DistroFamily::ArchFamily, "arch"),
        "ubuntu" => (DistroFamily::DebianFamily, "ubuntu"),
        "debian" => (DistroFamily::DebianFamily, "debian"),
        "linuxmint" | "mint" => (DistroFamily::DebianFamily, "mint"),
        "kali" => (DistroFamily::DebianFamily, "kali"),
        "parrot" => (DistroFamily::DebianFamily, "parrot"),
        "pop" => (DistroFamily::DebianFamily, "ubuntu"),
        "fedora" => (DistroFamily::FedoraFamily, "fedora"),
        "rhel" | "centos" | "rocky" | "alma" => (DistroFamily::FedoraFamily, "fedora"),
        "alpine" => (DistroFamily::AlpineFamily, "alpine"),
        "nixos" => (DistroFamily::NixFamily, "nixos"),
        "opensuse" | "opensuse-leap" | "opensuse-tumbleweed" =>
            (DistroFamily::SuseFamily, "opensuse"),
        _ => {
            if id_like.contains("arch") {
                (DistroFamily::ArchFamily, "arch")
            } else if id_like.contains("debian") || id_like.contains("ubuntu") {
                (DistroFamily::DebianFamily, "debian")
            } else if id_like.contains("fedora") || id_like.contains("rhel") {
                (DistroFamily::FedoraFamily, "fedora")
            } else if id_like.contains("suse") {
                (DistroFamily::SuseFamily, "opensuse")
            } else {
                (DistroFamily::Unknown, "linux")
            }
        }
    };

    DistroInfo {
        id,
        name,
        version,
        family,
        logo_name: logo_name.to_string(),
    }
}

pub fn detect_package_managers() -> Vec<Box<dyn PackageManager>> {
    let mut managers: Vec<Box<dyn PackageManager>> = Vec::new();

    let pacman = pmal::pacman::PacmanBackend::new();
    if pacman.detect() { managers.push(Box::new(pacman)); }

    let apt = pmal::apt::AptBackend::new();
    if apt.detect() { managers.push(Box::new(apt)); }

    let dnf = pmal::dnf::DnfBackend::new();
    if dnf.detect() { managers.push(Box::new(dnf)); }

    let apk = pmal::apk::ApkBackend::new();
    if apk.detect() { managers.push(Box::new(apk)); }

    let nix = pmal::nix::NixBackend::new();
    if nix.detect() { managers.push(Box::new(nix)); }

    let flatpak = pmal::flatpak::FlatpakBackend::new();
    if flatpak.detect() { managers.push(Box::new(flatpak)); }

    let snap = pmal::snap::SnapBackend::new();
    if snap.detect() { managers.push(Box::new(snap)); }

    // AppImage scanner is always available
    managers.push(Box::new(pmal::appimage::AppImageBackend::new()));

    managers
}
