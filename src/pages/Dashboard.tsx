import { useMemo } from "react";
import { useQuery } from "@tanstack/react-query";
import { Loader2 } from "lucide-react";
import { getAllPackages, getSystemInfo } from "../lib/commands";
import { formatBytes, formatUptime } from "../lib/format";
import { queryKeys } from "../lib/query-keys";
import { DistroLogo } from "../components/DistroLogo";
import { MetricGauge } from "../components/MetricGauge";
import { RefreshButton } from "../components/RefreshButton";
import { Button } from "../components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "../components/ui/card";

export function Dashboard() {
  const systemQuery = useQuery({
    queryKey: queryKeys.systemInfo,
    queryFn: getSystemInfo,
  });

  const packagesQuery = useQuery({
    queryKey: queryKeys.allPackages,
    queryFn: getAllPackages,
  });

  const isLoading = systemQuery.isLoading || packagesQuery.isLoading;
  const isError = systemQuery.isError || packagesQuery.isError;

  const stats = useMemo(() => {
    const packages = packagesQuery.data ?? [];
    return {
      totalInstalled: packages.length,
      orphans: packages.filter((pkg) => pkg.is_orphan).length,
      neverLaunched: packages.filter((pkg) => pkg.usage_tag === "never_launched").length,
      rarelyUsed: packages.filter((pkg) => pkg.usage_tag === "rarely_used").length,
    };
  }, [packagesQuery.data]);

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-16">
        <Loader2 className="animate-spin text-muted-foreground" size={32} />
      </div>
    );
  }

  if (isError || !systemQuery.data || !packagesQuery.data) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>Unable to load dashboard</CardTitle>
          <CardDescription>
            {(systemQuery.error instanceof Error && systemQuery.error.message) ||
              (packagesQuery.error instanceof Error && packagesQuery.error.message) ||
              "Unknown error"}
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Button
            onClick={() => {
              void Promise.all([systemQuery.refetch(), packagesQuery.refetch()]);
            }}
          >
            Retry
          </Button>
        </CardContent>
      </Card>
    );
  }

  const system = systemQuery.data;

  const cpuPercent = Math.min(100, Math.round(system.cpu_usage_percent));
  const ramPercent =
    system.ram_total_mb > 0
      ? Math.min(100, Math.round((system.ram_used_mb / system.ram_total_mb) * 100))
      : 0;
  const gpuPercent =
    system.gpu_vram_total_mb && system.gpu_vram_total_mb > 0
      ? Math.min(100, Math.round(((system.gpu_vram_used_mb ?? 0) / system.gpu_vram_total_mb) * 100))
      : null;

  return (
    <div className="space-y-5">
      <div className="flex items-center justify-between">
        <h1 className="text-3xl font-bold tracking-tight">Dashboard</h1>
        <RefreshButton queryKeys={[queryKeys.systemInfo, queryKeys.allPackages]} tooltip="Refresh system info and packages" />
      </div>

      {/* System Overview */}
      <Card>
        <CardHeader>
          <CardTitle>System Overview</CardTitle>
          <CardDescription>Current distro and runtime environment</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-3 mb-4">
            <DistroLogo
              distroName={system.distro_name}
              distroId={system.distro_id || system.distro_logo_name}
              distroIdLike={system.distro_id_like}
              size={40}
            />
            <div>
              <div className="font-semibold">{system.distro_name}</div>
              <div className="text-sm text-muted-foreground">{system.distro_version}</div>
            </div>
          </div>
          <dl className="grid grid-cols-2 gap-x-6 gap-y-2 text-sm sm:grid-cols-3">
            <div className="flex gap-2">
              <dt className="text-muted-foreground">Kernel</dt>
              <dd className="font-medium truncate">{system.kernel_version}</dd>
            </div>
            <div className="flex gap-2">
              <dt className="text-muted-foreground">Desktop / WM</dt>
              <dd className="font-medium truncate">{system.de_wm}</dd>
            </div>
            <div className="flex gap-2">
              <dt className="text-muted-foreground">Shell</dt>
              <dd className="font-medium truncate">{system.shell}</dd>
            </div>
            <div className="flex gap-2">
              <dt className="text-muted-foreground">Hostname</dt>
              <dd className="font-medium truncate">{system.hostname}</dd>
            </div>
            <div className="flex gap-2">
              <dt className="text-muted-foreground">User</dt>
              <dd className="font-medium truncate">{system.username}</dd>
            </div>
            <div className="flex gap-2">
              <dt className="text-muted-foreground">Uptime</dt>
              <dd className="font-medium">{formatUptime(system.uptime_seconds)}</dd>
            </div>
            <div className="flex gap-2">
              <dt className="text-muted-foreground">CPU</dt>
              <dd className="font-medium truncate">{system.cpu_model} ({system.cpu_cores} cores)</dd>
            </div>
            {system.gpu_name && (
              <div className="flex gap-2">
                <dt className="text-muted-foreground">GPU</dt>
                <dd className="font-medium truncate">{system.gpu_name}</dd>
              </div>
            )}
            {system.package_managers.length > 0 && (
              <div className="flex gap-2 col-span-2 sm:col-span-3">
                <dt className="text-muted-foreground shrink-0">Package Managers</dt>
                <dd className="font-medium truncate">{system.package_managers.map(m => m.charAt(0).toUpperCase() + m.slice(1)).join(", ")}</dd>
              </div>
            )}
          </dl>
        </CardContent>
      </Card>

      {/* System Metrics */}
      <div>
        <h2 className="mb-3 text-sm font-semibold text-muted-foreground uppercase tracking-wider">System Metrics</h2>
        <div className="flex flex-wrap gap-4">
          <MetricGauge
            value={cpuPercent}
            label="CPU"
            sublabel={`${system.cpu_model.split(" ").slice(-2).join(" ")} · ${system.cpu_cores}c`}
            color="hsl(35 85% 45%)"
          />
          <MetricGauge
            value={ramPercent}
            label="RAM"
            sublabel={`${(system.ram_used_mb / 1024).toFixed(1)} / ${(system.ram_total_mb / 1024).toFixed(1)} GB`}
            color="hsl(210 80% 55%)"
          />
          {gpuPercent !== null && system.gpu_name && (
            <MetricGauge
              value={gpuPercent}
              label="GPU VRAM"
              sublabel={`${system.gpu_vram_used_mb ?? 0} / ${system.gpu_vram_total_mb} MB`}
              color="hsl(270 70% 60%)"
            />
          )}
          {system.storage.map((mount) => {
            const usedPercent =
              mount.total_bytes > 0
                ? Math.min(100, Math.round((mount.used_bytes / mount.total_bytes) * 100))
                : 0;
            const label = mount.path === "/" ? "Storage (/)" : mount.path;
            return (
              <MetricGauge
                key={`${mount.path}-${mount.fs_type}`}
                value={usedPercent}
                label={label}
                sublabel={`${formatBytes(mount.used_bytes)} / ${formatBytes(mount.total_bytes)}`}
                color="hsl(160 60% 45%)"
              />
            );
          })}
          {/* Package Stats — same size as gauge cards */}
          <Card className="min-w-[160px] border-border/70">
            <CardContent className="flex flex-col items-center justify-center p-6">
              <div className="flex flex-col justify-between" style={{ width: 180, height: 180 }}>
                <div className="text-xs font-semibold uppercase tracking-wider text-muted-foreground text-center mb-2">Packages</div>
                <div className="flex-1 flex flex-col justify-evenly">
                  {[
                    { value: stats.totalInstalled, label: "Total" },
                    { value: stats.orphans, label: "Orphans" },
                    { value: stats.neverLaunched, label: "Never launched" },
                    { value: stats.rarelyUsed, label: "Rarely used" },
                  ].map(({ value, label }, i, arr) => (
                    <div
                      key={label}
                      className={`flex items-baseline justify-between gap-2${i < arr.length - 1 ? " pb-1.5 border-b border-border/50" : ""}`}
                    >
                      <span className="text-xs text-muted-foreground">{label}</span>
                      <span className="text-base font-bold tabular-nums leading-none">{value}</span>
                    </div>
                  ))}
                </div>
              </div>
            </CardContent>
          </Card>
        </div>
      </div>

      {/* Top 5 Packages by Size */}
      <Card>
        <CardHeader>
          <CardTitle>Top 5 Packages by Size</CardTitle>
          <CardDescription>Largest installed packages</CardDescription>
        </CardHeader>
        <CardContent>
          {system.top_packages_by_size.length === 0 ? (
            <p className="text-sm text-muted-foreground">No package size data available.</p>
          ) : (
            <ol className="space-y-2">
              {system.top_packages_by_size.slice(0, 5).map(([name, sizeBytes], i) => (
                <li key={name} className="flex items-center gap-3">
                  <span className="w-4 text-xs text-muted-foreground">{i + 1}.</span>
                  <span className="flex-1 font-mono text-sm">{name}</span>
                  <span className="font-mono text-sm text-muted-foreground">{formatBytes(sizeBytes)}</span>
                </li>
              ))}
            </ol>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
