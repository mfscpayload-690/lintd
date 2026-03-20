import { Package } from "lucide-react";

interface DistroLogoProps {
  distroName: string;
  size?: number;
}

interface LogoPalette {
  primary: string;
  secondary: string;
}

const distroPaletteMap: Record<string, LogoPalette> = {
  arch: { primary: "#1793d1", secondary: "#0b4f6c" },
  ubuntu: { primary: "#e95420", secondary: "#77216f" },
  fedora: { primary: "#294172", secondary: "#3c6eb4" },
  debian: { primary: "#a80030", secondary: "#d70a53" },
  kali: { primary: "#557c94", secondary: "#263d4d" },
  parrot: { primary: "#1fa84f", secondary: "#163b2f" },
  alpine: { primary: "#0d597f", secondary: "#2285b9" },
  nixos: { primary: "#5277c3", secondary: "#7ebae4" },
  manjaro: { primary: "#35bf5c", secondary: "#1d6f36" },
  garuda: { primary: "#6f4cc3", secondary: "#1f4ca1" },
  linuxmint: { primary: "#62a85f", secondary: "#274e3f" },
  opensuse: { primary: "#73ba25", secondary: "#173f1a" },
};

function normalizeDistroName(name: string): string {
  return name.toLowerCase().replace(/[^a-z]/g, "");
}

function resolvePalette(name: string): LogoPalette | null {
  const normalized = normalizeDistroName(name);
  const entries = Object.entries(distroPaletteMap);

  for (const [key, palette] of entries) {
    if (normalized.includes(key)) {
      return palette;
    }
  }

  return null;
}

export function DistroLogo({ distroName, size = 42 }: DistroLogoProps) {
  const palette = resolvePalette(distroName);

  if (!palette) {
    return (
      <div
        className="inline-flex items-center justify-center rounded-md border bg-muted"
        style={{ width: size, height: size }}
      >
        <Package className="h-5 w-5 text-muted-foreground" />
      </div>
    );
  }

  const center = size / 2;
  const outer = size * 0.45;
  const inner = size * 0.2;

  return (
    <svg
      width={size}
      height={size}
      viewBox={`0 0 ${size} ${size}`}
      role="img"
      aria-label={`${distroName} logo`}
      className="rounded-md border"
    >
      <rect width={size} height={size} fill="hsl(var(--card))" />
      <circle cx={center} cy={center} r={outer} fill={palette.primary} opacity={0.16} />
      <path
        d={`M ${center} ${size * 0.16} L ${size * 0.84} ${center} L ${center} ${size * 0.84} L ${size * 0.16} ${center} Z`}
        fill={palette.primary}
        opacity={0.9}
      />
      <circle cx={center} cy={center} r={inner} fill={palette.secondary} />
      <circle cx={center} cy={center} r={inner * 0.45} fill="white" opacity={0.9} />
    </svg>
  );
}
