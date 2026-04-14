import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import * as api from "../lib/tauri";
import type { ScanProgress } from "../lib/types";

export function useScan(projectId: string) {
  const [progress, setProgress] = useState<ScanProgress | null>(null);
  const qc = useQueryClient();

  const mutation = useMutation({
    mutationFn: () => api.startScan(projectId, setProgress),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["scans", projectId] });
    },
    onSettled: () => {
      setProgress(null);
    },
  });

  return { ...mutation, progress };
}

export function useScansForProject(projectId: string) {
  return useQuery({
    queryKey: ["scans", projectId],
    queryFn: () => api.listScansForProject(projectId),
    enabled: !!projectId,
  });
}

export function useScanResults(scanId: string) {
  return useQuery({
    queryKey: ["scanResults", scanId],
    queryFn: () => api.getScanResults(scanId),
    enabled: !!scanId,
  });
}
