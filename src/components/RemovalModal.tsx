import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Copy, Loader2 } from "lucide-react";
import { toast } from "sonner";
import { executeRemoval, previewRemoval } from "../lib/commands";
import { formatBytes } from "../lib/format";
import { queryKeys } from "../lib/query-keys";
import type { Package, RemovalPreview } from "../types/lintd";
import { Button } from "./ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "./ui/dialog";
import { ScrollArea } from "./ui/scroll-area";

interface RemovalModalProps {
  pkg: Package | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

function formatPreviewMessage(preview: RemovalPreview): string {
  return `Removed ${preview.package_name}. Recovered ${formatBytes(preview.size_to_recover_bytes)}.`;
}

export function RemovalModal({ pkg, open, onOpenChange }: RemovalModalProps) {
  const queryClient = useQueryClient();

  const previewQuery = useQuery({
    queryKey: pkg ? queryKeys.removalPreview(pkg.name, pkg.source) : ["removalPreview", "empty"],
    queryFn: async () => {
      if (!pkg) {
        throw new Error("No package selected");
      }
      return previewRemoval(pkg.name, pkg.source);
    },
    enabled: open && pkg !== null,
    retry: 1,
  });

  const executeMutation = useMutation({
    mutationFn: async () => {
      if (!pkg) {
        throw new Error("No package selected");
      }
      return executeRemoval(pkg.name, pkg.source);
    },
    onSuccess: async (result) => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: queryKeys.allPackages }),
        queryClient.invalidateQueries({ queryKey: queryKeys.orphans }),
        queryClient.invalidateQueries({ queryKey: queryKeys.removalHistory }),
      ]);

      const preview = previewQuery.data;
      const message =
        preview !== undefined
          ? formatPreviewMessage(preview)
          : `Removed ${result.package_name}. Recovered ${formatBytes(result.space_recovered_bytes)}.`;
      toast.success(message);
      onOpenChange(false);
    },
    onError: (error) => {
      const message = error instanceof Error ? error.message : "Failed to remove package";
      toast.error(message);
    },
  });

  const preview = previewQuery.data;
  const hasReverseDeps = (preview?.reverse_deps.length ?? 0) > 0;
  const systemCritical = preview?.is_system_critical ?? false;
  const confirmDisabled =
    executeMutation.isPending ||
    previewQuery.isLoading ||
    preview === undefined ||
    !preview.safe_to_remove ||
    hasReverseDeps;

  const copyCommand = async (): Promise<void> => {
    if (!preview) {
      return;
    }

    try {
      await navigator.clipboard.writeText(preview.cli_command_preview);
      toast.success("Command copied to clipboard");
    } catch {
      toast.error("Failed to copy command");
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl">
        {!pkg ? null : (
          <>
            <DialogHeader>
              <DialogTitle className="text-xl">Inspect & Remove</DialogTitle>
              <DialogDescription>
                Preview exactly what will happen before removing a package.
              </DialogDescription>
            </DialogHeader>

            {previewQuery.isLoading ? (
              <div className="flex items-center justify-center py-10 text-muted-foreground">
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                Loading removal preview...
              </div>
            ) : null}

            {previewQuery.isError ? (
              <div className="rounded-md border border-destructive/40 bg-destructive/10 p-3 text-sm text-destructive">
                {previewQuery.error instanceof Error
                  ? previewQuery.error.message
                  : "Failed to load removal preview."}
              </div>
            ) : null}

            {preview ? (
              <div className="space-y-4">
                <div>
                  <h3 className="text-lg font-semibold">{preview.package_name}</h3>
                  <p className="text-sm text-muted-foreground">{preview.description || "No description"}</p>
                </div>

                {hasReverseDeps ? (
                  <div className="border-l-2 border-destructive pl-3 py-2 text-sm text-destructive">
                    ⚠ Removing this may break: {preview.reverse_deps.join(", ")}
                  </div>
                ) : null}

                {systemCritical ? (
                  <div className="border-l-2 border-destructive pl-3 py-2 text-sm text-destructive">
                    System critical - removal blocked
                  </div>
                ) : null}

                <div>
                  <div className="mb-2 text-sm font-medium">Files to delete</div>
                  <ScrollArea className="h-[200px] rounded-md border bg-muted/20 p-0">
                    <div className="space-y-1">
                      {preview.files_to_delete.length === 0 ? (
                        <div className="font-mono text-xs leading-5 text-muted-foreground">No tracked files returned.</div>
                      ) : (
                        preview.files_to_delete.map((file) => <div key={file} className="font-mono text-xs leading-5">{file}</div>)
                      )}
                    </div>
                  </ScrollArea>
                </div>

                <div className="text-sm font-medium text-emerald-600 dark:text-emerald-400">
                  Space to recover: {formatBytes(preview.size_to_recover_bytes)}
                </div>

                <div className="space-y-2">
                  <div className="text-sm font-medium">Command preview</div>
                  <div className="flex items-start gap-2">
                    <pre className="max-h-24 flex-1 overflow-auto rounded-md border bg-muted/40 p-3 font-mono text-xs">
                      {preview.cli_command_preview}
                    </pre>
                    <Button type="button" size="sm" variant="ghost" onClick={copyCommand}>
                      <Copy className="h-4 w-4" />
                      Copy
                    </Button>
                  </div>
                </div>
              </div>
            ) : null}

            <DialogFooter>
              <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
                Cancel
              </Button>
              {!systemCritical ? (
                <Button
                  type="button"
                  variant="destructive"
                  disabled={confirmDisabled}
                  onClick={() => executeMutation.mutate()}
                >
                  {executeMutation.isPending ? (
                    <>
                      <Loader2 className="h-4 w-4 animate-spin" />
                      Removing...
                    </>
                  ) : (
                    "Confirm Removal"
                  )}
                </Button>
              ) : null}
            </DialogFooter>
          </>
        )}
      </DialogContent>
    </Dialog>
  );
}
