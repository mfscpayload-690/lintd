import { useMemo, useState } from "react";
import { Package } from "lucide-react";

interface DistroLogoProps {
  distroName: string;
  distroId?: string;
  distroIdLike?: string;
  size?: number;
}

const distroLogoMap: Record<string, string> = {
  arch: "arch",
  manjaro: "manjaro",
  endeavouros: "endeavouros",
  ubuntu: "ubuntu",
  debian: "debian",
  linuxmint: "linuxmint",
  mint: "linuxmint",
  kali: "kali",
  parrot: "parrot",
  pop: "pop",
  popos: "pop",
  fedora: "fedora",
  rhel: "fedora",
  centos: "fedora",
  rocky: "fedora",
  alma: "fedora",
  alpine: "alpine",
  nixos: "nixos",
  opensuse: "opensuse",
  opensuseleap: "opensuse",
  opensusetumbleweed: "opensuse",
};

const distroFamilyFallbackMap: Record<string, string> = {
  arch: "arch",
  debian: "debian",
  ubuntu: "ubuntu",
  fedora: "fedora",
  rhel: "fedora",
  suse: "opensuse",
  nix: "nixos",
  alpine: "alpine",
};

function normalizeId(value: string): string {
  return value.toLowerCase().replace(/[^a-z0-9]/g, "");
}

function resolveLogoKey(distroId?: string, distroIdLike?: string): string {
  const normalizedId = normalizeId(distroId ?? "");
  if (normalizedId.length > 0 && normalizedId in distroLogoMap) {
    return distroLogoMap[normalizedId];
  }

  const familyTokens = (distroIdLike ?? "")
    .split(/\s+/)
    .map((token) => normalizeId(token))
    .filter((token) => token.length > 0);

  for (const token of familyTokens) {
    if (token in distroFamilyFallbackMap) {
      return distroFamilyFallbackMap[token];
    }
  }

  return "linux";
}

export function DistroLogo({ distroName, distroId, distroIdLike, size = 42 }: DistroLogoProps) {
  const [hasError, setHasError] = useState(false);
  const logoKey = useMemo(
    () => resolveLogoKey(distroId, distroIdLike),
    [distroId, distroIdLike]
  );
  const src = `/distro-logos/${logoKey}.svg`;

  if (hasError) {
    return (
      <div
        className="inline-flex items-center justify-center rounded-md border bg-muted"
        style={{ width: size, height: size }}
      >
        <Package className="h-5 w-5 text-muted-foreground" />
      </div>
    );
  }

  return (
    <img
      src={src}
      alt={`${distroName} logo`}
      width={size}
      height={size}
      className="rounded-md border bg-card p-1 object-contain"
      onError={() => {
        setHasError(true);
      }}
    />
  );
}
