import { useQueryClient } from "@tanstack/react-query";
import { RefreshCw } from "lucide-react";
import { Button } from "./ui/button";
import { useCallback, useState } from "react";

interface RefreshButtonProps {
  queryKeys?: (string | readonly string[])[];
  onRefresh?: () => void | Promise<void>;
  tooltip?: string;
  disabled?: boolean;
}

export function RefreshButton({ queryKeys: keys, onRefresh, tooltip = "Refresh data", disabled }: RefreshButtonProps) {
  const queryClient = useQueryClient();
  const [isRefreshing, setIsRefreshing] = useState(false);

  const handleRefresh = useCallback(async () => {
    setIsRefreshing(true);
    try {
      if (onRefresh) {
        await onRefresh();
      } else if (keys) {
        await Promise.all(keys.map((key) => queryClient.invalidateQueries({ queryKey: Array.isArray(key) ? key : [key] })));
      }
    } finally {
      setIsRefreshing(false);
    }
  }, [queryClient, keys, onRefresh]);

  return (
    <Button
      variant="outline"
      size="sm"
      onClick={handleRefresh}
      disabled={isRefreshing || disabled}
      title={tooltip}
      className="gap-2"
    >
      <RefreshCw className={`h-4 w-4 ${isRefreshing ? "animate-spin" : ""}`} />
      <span className="hidden sm:inline">Refresh</span>
    </Button>
  );
}
