import { useMemo } from "react";
import { useQuery } from "@tanstack/react-query";
import { Bar, BarChart, CartesianGrid, ResponsiveContainer, XAxis, YAxis } from "recharts";
import { getAllPackages, getSystemInfo } from "../lib/commands";
import { formatBytes, formatUptime } from "../lib/format";
import { queryKeys } from "../lib/query-keys";
import { DistroLogo } from "../components/DistroLogo";
import { RefreshButton } from "../components/RefreshButton";
import { Button } from "../components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "../components/ui/card";
import { Progress } from "../components/ui/progress";

function LoadingSkeleton() {
  return (
    <div className="space-y-4">
      <div className="grid grid-cols-1 gap-4 lg:grid-cols-3">
        <div className="h-44 animate-pulse rounded-lg bg-muted" />
        <div className="h-44 animate-pulse rounded-lg bg-muted" />
        <div className="h-44 animate-pulse rounded-lg bg-muted" />
      </div>
      <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-4">
        <div className="h-28 animate-pulse rounded-lg bg-muted" />
        <div className="h-28 animate-pulse rounded-lg bg-muted" />
        <div className="h-28 animate-pulse rounded-lg bg-muted" />
        <div className="h-28 animate-pulse rounded-lg bg-muted" />
      </div>
      <div className="h-96 animate-pulse rounded-lg bg-muted" />
    </div>
  );
}

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

  const topPackages = useMemo(() => {
    const packages = packagesQuery.data ?? [];
    return [...packages]
      .sort((a, b) => b.size_bytes - a.size_bytes)
      .slice(0, 5)
      .map((pkg) => ({
        name: pkg.name,
        sizeMb: Number((pkg.size_bytes / (1024 * 1024)).toFixed(2)),
      }));
  }, [packagesQuery.data]);

  if (isLoading) {
    return <LoadingSkeleton />;
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
  const ramPercent =
    system.ram_total_mb > 0
      ? Math.min(100, Math.round((system.ram_used_mb / system.ram_total_mb) * 100))
      : 0;

  return (
    <div className="space-y-5">
      <div className="flex items-center justify-between">
        <h1 className="text-3xl font-bold tracking-tight">Dashboard</h1>
        <RefreshButton queryKeys={[queryKeys.systemInfo, queryKeys.allPackages]} tooltip="Refresh system info and packages" />
      </div>
      <div className="grid grid-cols-1 gap-4 lg:grid-cols-3">
        <Card className="lg:col-span-2">
          <CardHeader>
            <CardTitle>System Overview</CardTitle>
            <CardDescription>Current distro and runtime environment</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
              <div className="flex items-center gap-3 rounded-md border p-3">
                <DistroLogo
                  distroName={system.distro_name}
                  distroId={system.distro_id || system.distro_logo_name}
                  distroIdLike={system.distro_id_like}
                  size={46}
                />
                <div>
                  <div className="font-semibold">{system.distro_name}</div>
                  <div className="text-sm text-muted-foreground">{system.distro_version}</div>
                </div>
              </div>
              <div className="rounded-md border p-3">
                <div className="text-xs text-muted-foreground">Kernel</div>
                <div className="font-medium">{system.kernel_version}</div>
              </div>
              <div className="rounded-md border p-3">
                <div className="text-xs text-muted-foreground">Desktop / WM</div>
                <div className="font-medium">{system.de_wm}</div>
              </div>
              <div className="rounded-md border p-3">
                <div className="text-xs text-muted-foreground">Shell</div>
                <div className="font-medium">{system.shell}</div>
              </div>
              <div className="rounded-md border p-3">
                <div className="text-xs text-muted-foreground">Hostname</div>
                <div className="font-medium">{system.hostname}</div>
              </div>
              <div className="rounded-md border p-3">
                <div className="text-xs text-muted-foreground">Uptime</div>
                <div className="font-medium">{formatUptime(system.uptime_seconds)}</div>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Memory</CardTitle>
            <CardDescription>RAM usage</CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="text-2xl font-semibold">{ramPercent}%</div>
            <Progress value={ramPercent} />
            <div className="text-sm text-muted-foreground">
              {system.ram_used_mb} MB used / {system.ram_total_mb} MB total
            </div>
          </CardContent>
        </Card>
      </div>

      <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-4">
        <Card>
          <CardHeader className="pb-2">
            <CardDescription>Total Installed</CardDescription>
            <CardTitle className="text-2xl">{stats.totalInstalled}</CardTitle>
          </CardHeader>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardDescription>Orphans</CardDescription>
            <CardTitle className="text-2xl">{stats.orphans}</CardTitle>
          </CardHeader>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardDescription>Never Launched</CardDescription>
            <CardTitle className="text-2xl">{stats.neverLaunched}</CardTitle>
          </CardHeader>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardDescription>Rarely Used</CardDescription>
            <CardTitle className="text-2xl">{stats.rarelyUsed}</CardTitle>
          </CardHeader>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Storage</CardTitle>
          <CardDescription>Usage by mount point</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 gap-3 lg:grid-cols-2">
            {system.storage.map((mount) => {
              const usedPercent =
                mount.total_bytes > 0
                  ? Math.min(100, Math.round((mount.used_bytes / mount.total_bytes) * 100))
                  : 0;

              return (
                <Card key={`${mount.path}-${mount.fs_type}`}>
                  <CardHeader className="pb-3">
                    <CardTitle className="text-base">{mount.path}</CardTitle>
                    <CardDescription>{mount.fs_type}</CardDescription>
                  </CardHeader>
                  <CardContent className="space-y-2">
                    <Progress value={usedPercent} />
                    <div className="text-sm text-muted-foreground">
                      {formatBytes(mount.used_bytes)} used / {formatBytes(mount.total_bytes)} total
                    </div>
                    <div className="text-sm text-muted-foreground">
                      {formatBytes(mount.free_bytes)} free
                    </div>
                  </CardContent>
                </Card>
              );
            })}
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Top 5 Packages by Size</CardTitle>
          <CardDescription>Largest installed packages</CardDescription>
        </CardHeader>
        <CardContent className="h-[320px]">
          {topPackages.length === 0 ? (
            <div className="flex h-full items-center justify-center text-sm text-muted-foreground">
              No packages available to visualize.
            </div>
          ) : (
            <ResponsiveContainer width="100%" height="100%">
              <BarChart data={topPackages} layout="vertical" margin={{ left: 24, right: 16 }}>
                <CartesianGrid strokeDasharray="3 3" />
                <YAxis dataKey="name" type="category" width={140} />
                <XAxis dataKey="sizeMb" type="number" unit=" MB" />
                <Bar dataKey="sizeMb" fill="hsl(var(--primary))" radius={[4, 4, 4, 4]} />
              </BarChart>
            </ResponsiveContainer>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
