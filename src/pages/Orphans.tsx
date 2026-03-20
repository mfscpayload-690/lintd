import { useMemo, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Loader2, TriangleAlert } from "lucide-react";
import { toast } from "sonner";
import { executeRemoval, getOrphans, previewRemoval } from "../lib/commands";
import { formatBytes, formatDate } from "../lib/format";
import { sourceBadgeClassMap, sourceLabelMap, usageBadgeClassMap, usageLabelMap } from "../lib/presentation";
import { queryKeys } from "../lib/query-keys";
import type { Package, RemovalPreview } from "../types/lintd";
import { RemovalModal } from "../components/RemovalModal";
import { Badge } from "../components/ui/badge";
import { Button } from "../components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "../components/ui/dialog";
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

interface RemoveAllSummary {
  previews: RemovalPreview[];
  totalSafeSpace: number;
}

function LoadingRows() {
  return (
    <>
      {Array.from({ length: 8 }).map((_, index) => (
        <TableRow key={index}>
          <TableCell colSpan={9}>
            <div className="h-5 w-full animate-pulse rounded bg-muted" />
          </TableCell>
        </TableRow>
      ))}
    </>
  );
}

export function Orphans() {
  const queryClient = useQueryClient();
  const [selectedPackage, setSelectedPackage] = useState<Package | null>(null);
  const [page, setPage] = useState(1);
  const [summary, setSummary] = useState<RemoveAllSummary | null>(null);
  const [summaryOpen, setSummaryOpen] = useState(false);

  const orphansQuery = useQuery({
    queryKey: queryKeys.orphans,
    queryFn: getOrphans,
  });

  const removeAllPreviewMutation = useMutation({
    mutationFn: async (orphans: Package[]): Promise<RemoveAllSummary> => {
      const previews: RemovalPreview[] = [];
      for (const orphan of orphans) {
        const preview = await previewRemoval(orphan.name, orphan.source);
        previews.push(preview);
      }

      const totalSafeSpace = previews
        .filter((preview) => preview.safe_to_remove && !preview.is_system_critical)
        .reduce((sum, preview) => sum + preview.size_to_recover_bytes, 0);

      return { previews, totalSafeSpace };
    },
    onSuccess: (data) => {
      setSummary(data);
      setSummaryOpen(true);
    },
    onError: (error) => {
      toast.error(error instanceof Error ? error.message : "Failed to build removal summary");
    },
  });

  const orphans = orphansQuery.data ?? [];
  const pageRows = useMemo(() => {
    const totalPages = Math.max(1, Math.ceil(orphans.length / PAGE_SIZE));
    const currentPage = Math.min(page, totalPages);
    return orphans.slice((currentPage - 1) * PAGE_SIZE, currentPage * PAGE_SIZE);
  }, [orphans, page]);

  const totalPages = Math.max(1, Math.ceil(orphans.length / PAGE_SIZE));
  const currentPage = Math.min(page, totalPages);

  const confirmRemoveAll = async (): Promise<void> => {
    if (!summary) {
      return;
    }

    const safePreviews = summary.previews.filter(
      (preview) =>
        preview.safe_to_remove && !preview.is_system_critical && preview.reverse_deps.length === 0
    );

    for (let index = 0; index < safePreviews.length; index += 1) {
      const preview = safePreviews[index];
      const matched = orphans.find((orphan) => orphan.name === preview.package_name);
      if (!matched) {
        continue;
      }

      const loadingId = `remove-orphan-${preview.package_name}`;
      toast.loading(`Removing ${preview.package_name} (${index + 1}/${safePreviews.length})`, {
        id: loadingId,
      });
      try {
        const result = await executeRemoval(matched.name, matched.source);
        toast.success(
          `${result.package_name} removed (${formatBytes(result.space_recovered_bytes)} recovered)`,
          { id: loadingId }
        );
      } catch (error) {
        toast.error(
          error instanceof Error ? error.message : `Failed to remove ${preview.package_name}`,
          { id: loadingId }
        );
      }
    }

    await Promise.all([
      queryClient.invalidateQueries({ queryKey: queryKeys.allPackages }),
      queryClient.invalidateQueries({ queryKey: queryKeys.orphans }),
      queryClient.invalidateQueries({ queryKey: queryKeys.removalHistory }),
    ]);

    setSummaryOpen(false);
    setSummary(null);
  };

  return (
    <div className="space-y-4">
      <Card>
        <CardContent className="pt-6">
          <div className="flex flex-col gap-3 rounded-md border border-amber-500/30 bg-amber-50 p-3 text-sm text-amber-900 dark:bg-amber-950/40 dark:text-amber-300 md:flex-row md:items-center md:justify-between">
            <div className="flex items-start gap-2">
              <TriangleAlert className="mt-0.5 h-4 w-4" />
              <span>
                Orphan packages are dependencies no longer required by installed apps. They are
                generally safe to remove after review.
              </span>
            </div>
            <Button
              variant="destructive"
              disabled={orphans.length === 0 || removeAllPreviewMutation.isPending}
              onClick={() => removeAllPreviewMutation.mutate(orphans)}
            >
              {removeAllPreviewMutation.isPending ? (
                <>
                  <Loader2 className="h-4 w-4 animate-spin" />
                  Building summary...
                </>
              ) : (
                "Remove All Orphans"
              )}
            </Button>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Orphan Packages</CardTitle>
        </CardHeader>
        <CardContent>
          {orphansQuery.isLoading ? (
            <Table>
              <TableBody>
                <LoadingRows />
              </TableBody>
            </Table>
          ) : null}

          {orphansQuery.isError ? (
            <div className="rounded-md border border-destructive/40 bg-destructive/10 p-3 text-sm text-destructive">
              <div className="mb-2">
                {orphansQuery.error instanceof Error
                  ? orphansQuery.error.message
                  : "Failed to load orphan packages"}
              </div>
              <Button size="sm" onClick={() => void orphansQuery.refetch()}>
                Retry
              </Button>
            </div>
          ) : null}

          {!orphansQuery.isLoading && !orphansQuery.isError && orphans.length === 0 ? (
            <div className="rounded-md border border-dashed p-8 text-center text-muted-foreground">
              No orphan packages found 🎉
            </div>
          ) : null}

          {!orphansQuery.isLoading && !orphansQuery.isError && orphans.length > 0 ? (
            <>
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Name</TableHead>
                    <TableHead>Description</TableHead>
                    <TableHead>Source</TableHead>
                    <TableHead>Version</TableHead>
                    <TableHead>Size</TableHead>
                    <TableHead>Install Date</TableHead>
                    <TableHead>Last Used</TableHead>
                    <TableHead>Usage Tag</TableHead>
                    <TableHead className="text-right">Action</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {pageRows.map((pkg) => (
                    <TableRow key={`${pkg.source}:${pkg.name}`}>
                      <TableCell className="font-medium">{pkg.name}</TableCell>
                      <TableCell className="max-w-[260px] truncate">{pkg.description || "-"}</TableCell>
                      <TableCell>
                        <Badge className={cn("border-0", sourceBadgeClassMap[pkg.source])}>
                          {sourceLabelMap[pkg.source]}
                        </Badge>
                      </TableCell>
                      <TableCell>{pkg.version}</TableCell>
                      <TableCell>{formatBytes(pkg.size_bytes)}</TableCell>
                      <TableCell>{formatDate(pkg.install_date)}</TableCell>
                      <TableCell>{formatDate(pkg.last_used)}</TableCell>
                      <TableCell>
                        <Badge className={cn("border-0", usageBadgeClassMap[pkg.usage_tag])}>
                          {usageLabelMap[pkg.usage_tag]}
                        </Badge>
                      </TableCell>
                      <TableCell className="text-right">
                        <Button size="sm" variant="outline" onClick={() => setSelectedPackage(pkg)}>
                          Inspect & Remove
                        </Button>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>

              <div className="mt-4 flex items-center justify-between">
                <div className="text-sm text-muted-foreground">
                  Showing {(currentPage - 1) * PAGE_SIZE + 1}-
                  {(currentPage - 1) * PAGE_SIZE + pageRows.length} of {orphans.length}
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
            </>
          ) : null}
        </CardContent>
      </Card>

      <Dialog open={summaryOpen} onOpenChange={setSummaryOpen}>
        <DialogContent className="max-w-2xl">
          <DialogHeader>
            <DialogTitle>Remove All Orphans</DialogTitle>
            <DialogDescription>
              Review packages queued for removal and confirm once.
            </DialogDescription>
          </DialogHeader>
          {summary ? (
            <div className="space-y-3">
              <div className="rounded-md border p-3 text-sm">
                Potential recovered space: <span className="font-semibold">{formatBytes(summary.totalSafeSpace)}</span>
              </div>
              <div className="max-h-64 overflow-auto rounded-md border">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>Package</TableHead>
                      <TableHead>Recoverable</TableHead>
                      <TableHead>Status</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {summary.previews.map((preview) => {
                      const safe =
                        preview.safe_to_remove &&
                        !preview.is_system_critical &&
                        preview.reverse_deps.length === 0;
                      return (
                        <TableRow key={preview.package_name}>
                          <TableCell>{preview.package_name}</TableCell>
                          <TableCell>{formatBytes(preview.size_to_recover_bytes)}</TableCell>
                          <TableCell>
                            <Badge
                              className={cn(
                                "border-0",
                                safe
                                  ? "bg-emerald-100 text-emerald-800 dark:bg-emerald-950 dark:text-emerald-300"
                                  : "bg-red-100 text-red-800 dark:bg-red-950 dark:text-red-300"
                              )}
                            >
                              {safe ? "Safe" : "Blocked"}
                            </Badge>
                          </TableCell>
                        </TableRow>
                      );
                    })}
                  </TableBody>
                </Table>
              </div>
            </div>
          ) : null}
          <DialogFooter>
            <Button variant="outline" onClick={() => setSummaryOpen(false)}>
              Cancel
            </Button>
            <Button variant="destructive" onClick={() => void confirmRemoveAll()}>
              Confirm Removal
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

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
