import { useEffect, useMemo, useState } from "react";
import { AlertCircle, ArrowDown, ArrowUp, ArrowUpDown, Search } from "lucide-react";
import { formatBytes, formatDate } from "../lib/format";
import {
  PACKAGE_SOURCES,
  USAGE_TAGS,
  sourceBadgeClassMap,
  sourceLabelMap,
  usageBadgeClassMap,
  usageLabelMap,
} from "../lib/presentation";
import { useDebouncedValue } from "../lib/use-debounced-value";
import { useStreamingScan } from "../lib/use-streaming-scan";
import type { Package, PackageSource, UsageTag } from "../types/lintd";
import { RefreshButton } from "../components/RefreshButton";
import { RemovalModal } from "../components/RemovalModal";
import { Badge } from "../components/ui/badge";
import { Button } from "../components/ui/button";
import { Card, CardContent } from "../components/ui/card";
import { Input } from "../components/ui/input";
import { Progress } from "../components/ui/progress";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "../components/ui/table";
import { cn } from "../lib/utils";

const PAGE_SIZE = 25;

type SortKey =
  | "name"
  | "description"
  | "source"
  | "version"
  | "size_bytes"
  | "install_date"
  | "last_used"
  | "usage_tag";

type SortDirection = "asc" | "desc";

function SortIcon({ active, direction }: { active: boolean; direction: SortDirection }) {
  if (!active) {
    return <ArrowUpDown className="h-3.5 w-3.5" />;
  }
  return direction === "asc" ? (
    <ArrowUp className="h-3.5 w-3.5" />
  ) : (
    <ArrowDown className="h-3.5 w-3.5" />
  );
}

function compareValue(pkg: Package, key: SortKey): string | number {
  switch (key) {
    case "size_bytes":
      return pkg.size_bytes;
    case "install_date":
      return pkg.install_date ? new Date(pkg.install_date).getTime() : 0;
    case "last_used":
      return pkg.last_used ? new Date(pkg.last_used).getTime() : 0;
    default:
      return String(pkg[key] ?? "").toLowerCase();
  }
}

function isLikelyCritical(pkg: Package): boolean {
  const name = pkg.name.toLowerCase();
  const exactMatches = new Set([
    "linux",
    "linux-lts",
    "linux-zen",
    "linux-hardened",
    "linux-headers",
    "glibc",
    "systemd",
    "webkit2gtk",
    "polkit",
    "dbus",
    "sudo",
    "bash",
    "pacman",
    "dpkg",
    "apt",
    "dnf",
  ]);

  if (exactMatches.has(name)) {
    return true;
  }

  if (name.startsWith("nvidia-") && (name.includes("dkms") || name.includes("open"))) {
    return true;
  }

  return false;
}

export function Packages() {
  const [searchInput, setSearchInput] = useState("");
  const [sourceFilter, setSourceFilter] = useState<PackageSource | "all">("all");
  const [usageFilter, setUsageFilter] = useState<UsageTag | "all">("all");
  const [sortKey, setSortKey] = useState<SortKey>("size_bytes");
  const [sortDirection, setSortDirection] = useState<SortDirection>("desc");
  const [page, setPage] = useState(1);
  const [selectedPackage, setSelectedPackage] = useState<Package | null>(null);

  const debouncedSearch = useDebouncedValue(searchInput, 300).trim().toLowerCase();

  const { packages, isScanning, progress, managersDone, managersTotal, errors, startScan } =
    useStreamingScan();

  // Start scan on mount
  useEffect(() => {
    void startScan();
  }, [startScan]);

  const filteredAndSorted = useMemo(() => {
    const filtered = packages.filter((pkg) => {
      if (debouncedSearch.length > 0 && !pkg.name.toLowerCase().includes(debouncedSearch)) {
        return false;
      }
      if (sourceFilter !== "all" && pkg.source !== sourceFilter) {
        return false;
      }
      if (usageFilter !== "all" && pkg.usage_tag !== usageFilter) {
        return false;
      }
      return true;
    });

    filtered.sort((a, b) => {
      const aValue = compareValue(a, sortKey);
      const bValue = compareValue(b, sortKey);
      if (aValue < bValue) {
        return sortDirection === "asc" ? -1 : 1;
      }
      if (aValue > bValue) {
        return sortDirection === "asc" ? 1 : -1;
      }
      return 0;
    });

    return filtered;
  }, [debouncedSearch, packages, sortDirection, sortKey, sourceFilter, usageFilter]);

  const totalPages = Math.max(1, Math.ceil(filteredAndSorted.length / PAGE_SIZE));
  const currentPage = Math.min(page, totalPages);
  const pageRows = filteredAndSorted.slice((currentPage - 1) * PAGE_SIZE, currentPage * PAGE_SIZE);

  const onSort = (key: SortKey): void => {
    setPage(1);
    if (sortKey === key) {
      setSortDirection((prev) => (prev === "asc" ? "desc" : "asc"));
      return;
    }
    setSortKey(key);
    setSortDirection("asc");
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h1 className="text-3xl font-bold tracking-tight">Packages</h1>
        <RefreshButton
          onRefresh={startScan}
          disabled={isScanning}
          tooltip="Refresh package list"
        />
      </div>
      <Card>
        <CardContent>
          <div className="grid grid-cols-1 gap-3 md:grid-cols-3">
            <div className="relative">
              <Search className="pointer-events-none absolute left-3 top-3 h-4 w-4 text-muted-foreground" />
              <Input
                placeholder="Search package name"
                className="pl-9"
                value={searchInput}
                onChange={(event) => {
                  setSearchInput(event.target.value);
                  setPage(1);
                }}
              />
            </div>

            <select
              className="h-10 rounded-md border bg-background px-3 text-sm"
              value={sourceFilter}
              onChange={(event) => {
                setSourceFilter(event.target.value as PackageSource | "all");
                setPage(1);
              }}
            >
              <option value="all">All Sources</option>
              {PACKAGE_SOURCES.map((source) => (
                <option key={source} value={source}>
                  {sourceLabelMap[source]}
                </option>
              ))}
            </select>

            <select
              className="h-10 rounded-md border bg-background px-3 text-sm"
              value={usageFilter}
              onChange={(event) => {
                setUsageFilter(event.target.value as UsageTag | "all");
                setPage(1);
              }}
            >
              <option value="all">All Usage Tags</option>
              {USAGE_TAGS.map((tag) => (
                <option key={tag} value={tag}>
                  {usageLabelMap[tag]}
                </option>
              ))}
            </select>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardContent className="pt-3">
          {/* Scan progress bar */}
          {isScanning && (
            <div className="mb-4 space-y-1.5">
              <div className="flex items-center gap-2">
                <span className="text-sm text-muted-foreground">
                  Scanning: {managersDone}/{managersTotal} managers
                </span>
                {errors.length > 0 && (
                  <div className="flex items-center gap-1">
                    {errors.map((err, i) => (
                      <span
                        key={i}
                        className="inline-flex items-center gap-1 rounded border border-destructive/40 bg-destructive/10 px-1.5 py-0.5 text-xs text-destructive"
                        title={err}
                      >
                        <AlertCircle className="h-3 w-3" />
                        Error
                      </span>
                    ))}
                  </div>
                )}
              </div>
              <Progress value={progress} className="h-1.5" />
            </div>
          )}

          <Table>
            <TableHeader>
              <TableRow>
                {[
                  { key: "name", label: "Name" },
                  { key: "description", label: "Description" },
                  { key: "source", label: "Source" },
                  { key: "version", label: "Version" },
                  { key: "size_bytes", label: "Size" },
                  { key: "install_date", label: "Install Date" },
                  { key: "last_used", label: "Last Used" },
                  { key: "usage_tag", label: "Usage Tag" },
                ].map((column) => {
                  const key = column.key as SortKey;
                  return (
                    <TableHead key={column.key}>
                      <button
                        type="button"
                        className="inline-flex items-center gap-1 font-medium"
                        onClick={() => onSort(key)}
                      >
                        {column.label}
                        <SortIcon active={sortKey === key} direction={sortDirection} />
                      </button>
                    </TableHead>
                  );
                })}
                <TableHead className="text-right">Action</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {isScanning && packages.length === 0 ? (
                <TableRow>
                  <TableCell colSpan={9} className="text-center text-muted-foreground">
                    Scanning packages…
                  </TableCell>
                </TableRow>
              ) : null}

              {!isScanning && packages.length === 0 && errors.length > 0 ? (
                <TableRow>
                  <TableCell colSpan={9}>
                    <div className="flex items-center justify-between rounded-md border border-destructive/40 bg-destructive/10 p-3">
                      <span className="text-sm text-destructive">
                        {errors[0]}
                      </span>
                      <Button size="sm" onClick={() => void startScan()}>
                        Retry
                      </Button>
                    </div>
                  </TableCell>
                </TableRow>
              ) : null}

              {packages.length > 0 && pageRows.length === 0 ? (
                <TableRow>
                  <TableCell colSpan={9} className="text-center text-muted-foreground">
                    No packages match the selected filters.
                  </TableCell>
                </TableRow>
              ) : null}

              {pageRows.map((pkg) => (
                <TableRow key={`${pkg.source}:${pkg.name}`} className="h-8 font-mono text-xs">
                  <TableCell className="font-medium font-mono">{pkg.name}</TableCell>
                  <TableCell className="max-w-[260px] truncate">{pkg.description || "-"}</TableCell>
                  <TableCell>
                    <Badge className={cn("border-0", sourceBadgeClassMap[pkg.source])}>
                      {sourceLabelMap[pkg.source]}
                    </Badge>
                  </TableCell>
                  <TableCell className="font-mono">{pkg.version}</TableCell>
                  <TableCell className="font-mono">{formatBytes(pkg.size_bytes)}</TableCell>
                  <TableCell className="font-mono">{formatDate(pkg.install_date)}</TableCell>
                  <TableCell className="font-mono">{formatDate(pkg.last_used)}</TableCell>
                  <TableCell>
                    <Badge className={cn("border-0", usageBadgeClassMap[pkg.usage_tag])}>
                      {usageLabelMap[pkg.usage_tag]}
                    </Badge>
                  </TableCell>
                  <TableCell className="text-right">
                    {isLikelyCritical(pkg) ? (
                      <Button size="sm" variant="outline" disabled className="cursor-not-allowed opacity-50">
                        System Package
                      </Button>
                    ) : (
                      <Button size="sm" variant="outline" onClick={() => setSelectedPackage(pkg)}>
                        Inspect & Remove
                      </Button>
                    )}
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>

          <div className="mt-4 flex items-center justify-between">
            <div className="text-sm text-muted-foreground">
              Showing {(currentPage - 1) * PAGE_SIZE + (pageRows.length > 0 ? 1 : 0)}-
              {(currentPage - 1) * PAGE_SIZE + pageRows.length} of {filteredAndSorted.length}
            </div>
            <div className="flex items-center gap-2">
              <Button
                size="sm"
                variant="outline"
                disabled={currentPage <= 1}
                onClick={() => setPage((prev) => Math.max(1, prev - 1))}
              >
                Previous
              </Button>
              <span className="text-sm">
                Page {currentPage} / {totalPages}
              </span>
              <Button
                size="sm"
                variant="outline"
                disabled={currentPage >= totalPages}
                onClick={() => setPage((prev) => Math.min(totalPages, prev + 1))}
              >
                Next
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>

      <RemovalModal
        pkg={selectedPackage}
        open={selectedPackage !== null}
        onOpenChange={(open) => {
          if (!open) {
            setSelectedPackage(null);
          }
        }}
      />
    </div>
  );
}
