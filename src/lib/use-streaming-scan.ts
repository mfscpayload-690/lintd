import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { scanPackagesStreaming } from './commands';
import type { Package, ScanProgressEvent } from '../types/lintd';

interface StreamingScanState {
  isScanning: boolean;
  managersDone: number;
  managersTotal: number;
  errors: string[];
}

const INITIAL_STATE: StreamingScanState = {
  isScanning: false,
  managersDone: 0,
  managersTotal: 0,
  errors: [],
};

export function useStreamingScan() {
  const packagesRef = useRef<Record<string, Package[]>>({});
  const unlistenRef = useRef<UnlistenFn | null>(null);
  const [state, setState] = useState<StreamingScanState>(INITIAL_STATE);

  useEffect(() => {
    return () => {
      unlistenRef.current?.();
    };
  }, []);

  const startScan = useCallback(async () => {
    // Clean up any previous listener
    unlistenRef.current?.();
    unlistenRef.current = null;

    // Reset accumulated packages and state
    packagesRef.current = {};
    setState({ isScanning: true, managersDone: 0, managersTotal: 0, errors: [] });

    const unlisten = await listen<ScanProgressEvent>('scan_progress', (event) => {
      const { source, packages, done_count, total_count, error } = event.payload;

      packagesRef.current = {
        ...packagesRef.current,
        [source]: packages,
      };

      setState((prev) => {
        const errors = error ? [...prev.errors, error] : prev.errors;
        return {
          isScanning: done_count !== total_count,
          managersDone: done_count,
          managersTotal: total_count,
          errors,
        };
      });
    });

    unlistenRef.current = unlisten;

    await scanPackagesStreaming();
  }, []);

  const reset = useCallback(() => {
    unlistenRef.current?.();
    unlistenRef.current = null;
    packagesRef.current = {};
    setState(INITIAL_STATE);
  }, []);

  const packages = useMemo(
    () => Object.values(packagesRef.current).flat(),
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [state.managersDone, state.managersTotal]
  );

  const progress = useMemo(() => {
    if (state.managersTotal <= 0) return 0;
    return Math.min(100, Math.max(0, (state.managersDone / state.managersTotal) * 100));
  }, [state.managersDone, state.managersTotal]);

  return {
    packages,
    isScanning: state.isScanning,
    progress,
    managersDone: state.managersDone,
    managersTotal: state.managersTotal,
    errors: state.errors,
    startScan,
    reset,
  };
}
