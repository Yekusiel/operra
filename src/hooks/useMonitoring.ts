import { useQuery } from "@tanstack/react-query";
import * as api from "../lib/tauri";

const REFRESH_INTERVAL = 60_000; // 60 seconds

export function useInstanceStatus(projectId: string, enabled = true) {
  return useQuery({
    queryKey: ["monitoring", "instance", projectId],
    queryFn: () => api.getInstanceStatus(projectId),
    enabled: !!projectId && enabled,
    refetchInterval: enabled ? REFRESH_INTERVAL : false,
    retry: 1,
  });
}

export function useAppHealth(projectId: string, enabled = true) {
  return useQuery({
    queryKey: ["monitoring", "health", projectId],
    queryFn: () => api.getAppHealth(projectId),
    enabled: !!projectId && enabled,
    refetchInterval: enabled ? REFRESH_INTERVAL : false,
    retry: 1,
  });
}

export function useCloudWatchMetrics(
  projectId: string,
  metricName: string,
  hours: number,
  enabled = true
) {
  return useQuery({
    queryKey: ["monitoring", "metrics", projectId, metricName, hours],
    queryFn: () => api.getCloudWatchMetrics(projectId, metricName, hours),
    enabled: !!projectId && enabled,
    refetchInterval: enabled ? REFRESH_INTERVAL : false,
    retry: 1,
  });
}

export function useContainerStatus(projectId: string, enabled = true) {
  return useQuery({
    queryKey: ["monitoring", "containers", projectId],
    queryFn: () => api.getContainerStatus(projectId),
    enabled: !!projectId && enabled,
    refetchInterval: enabled ? REFRESH_INTERVAL : false,
    retry: 1,
  });
}

export function useCostSummary(projectId: string, enabled = true) {
  return useQuery({
    queryKey: ["monitoring", "cost", projectId],
    queryFn: () => api.getCostSummary(projectId),
    enabled: !!projectId && enabled,
    refetchInterval: enabled ? REFRESH_INTERVAL * 5 : false, // Cost data refreshes slower
    retry: 1,
  });
}
