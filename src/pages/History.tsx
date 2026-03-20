import { useMemo } from "react";
import { useQuery } from "@tanstack/react-query";
import { getRemovalHistory } from "../lib/commands";
import { formatBytes, formatDate } from "../lib/format";
import { sourceBadgeClassMap, sourceLabelMap } from "../lib/presentation";
import { queryKeys } from "../lib/query-keys";
import { RefreshButton } from "../components/RefreshButton";
import { Badge } from "../components/ui/badge";
import { Button } from "../components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "../components/ui/table";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "../components/ui/tooltip";
import { cn } from "../lib/utils";

function LoadingRows() {
  return (
    <>
      {Array.from({ length: 10 }).map((_, index) => (
        <TableRow key={index}>
          <TableCell colSpan={5}>
            <div className="h-5 w-full animate-pulse rounded bg-muted" />
          </TableCell>
        </TableRow>
      ))}
    </>
  );
}

export function History() {
  const historyQuery = useQuery({
    queryKey: queryKeys.removalHistory,
    queryFn: getRemovalHistory,
  });

  const totalRecovered = useMemo(() => {
    const history = historyQuery.data ?? [];
    return history.reduce((sum, record) => sum + record.space_recovered_bytes, 0);
  }, [historyQuery.data]);

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h1 className="text-3xl font-bold tracking-tight">History</h1>
        <RefreshButton queryKeys={[queryKeys.removalHistory]} tooltip="Refresh removal history" />
      </div>
      <Card>
        <CardHeader>
          <CardTitle>Total Space Recovered</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="text-3xl font-semibold">{formatBytes(totalRecovered)}</div>
        </CardContent>
      </Card>

      <Card>
        <CardContent className="pt-6">
          <TooltipProvider>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Package Name</TableHead>
                  <TableHead>Source</TableHead>
                  <TableHead>Date Removed</TableHead>
                  <TableHead>Space Recovered</TableHead>
                  <TableHead>Command Run</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {historyQuery.isLoading ? <LoadingRows /> : null}

                {historyQuery.isError ? (
                  <TableRow>
                    <TableCell colSpan={5}>
                      <div className="flex items-center justify-between rounded-md border border-destructive/40 bg-destructive/10 p-3">
                        <span className="text-sm text-destructive">
                          {historyQuery.error instanceof Error
                            ? historyQuery.error.message
                            : "Failed to load history"}
                        </span>
                        <Button size="sm" onClick={() => void historyQuery.refetch()}>
                          Retry
                        </Button>
                      </div>
                    </TableCell>
                  </TableRow>
                ) : null}

                {!historyQuery.isLoading && !historyQuery.isError && (historyQuery.data?.length ?? 0) === 0 ? (
                  <TableRow>
                    <TableCell colSpan={5} className="text-center text-muted-foreground">
                      No packages removed yet
                    </TableCell>
                  </TableRow>
                ) : null}

                {!historyQuery.isLoading &&
                  !historyQuery.isError &&
                  historyQuery.data?.map((record) => (
                    <TableRow key={record.id}>
                      <TableCell className="font-medium">{record.package_name}</TableCell>
                      <TableCell>
                        <Badge className={cn("border-0", sourceBadgeClassMap[record.source])}>
                          {sourceLabelMap[record.source]}
                        </Badge>
                      </TableCell>
                      <TableCell>{formatDate(record.removed_at)}</TableCell>
                      <TableCell>{formatBytes(record.space_recovered_bytes)}</TableCell>
                      <TableCell className="max-w-[360px]">
                        <Tooltip>
                          <TooltipTrigger asChild>
                            <span className="block truncate font-mono text-xs">{record.command_executed}</span>
                          </TooltipTrigger>
                          <TooltipContent className="max-w-[500px] break-all font-mono text-xs">
                            {record.command_executed}
                          </TooltipContent>
                        </Tooltip>
                      </TableCell>
                    </TableRow>
                  ))}
              </TableBody>
            </Table>
          </TooltipProvider>
        </CardContent>
      </Card>
    </div>
  );
}
