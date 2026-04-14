import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import * as api from "../lib/tauri";

export function useLatestPlan(projectId: string) {
  return useQuery({
    queryKey: ["latestPlan", projectId],
    queryFn: () => api.getLatestPlan(projectId),
    enabled: !!projectId,
  });
}

export function usePlan(planId: string) {
  return useQuery({
    queryKey: ["plan", planId],
    queryFn: () => api.getPlan(planId),
    enabled: !!planId,
  });
}

export function usePlans(projectId: string) {
  return useQuery({
    queryKey: ["plans", projectId],
    queryFn: () => api.listPlans(projectId),
    enabled: !!projectId,
  });
}

export function useGeneratePlan(projectId: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: () => api.generatePlan(projectId),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["latestPlan", projectId] });
      qc.invalidateQueries({ queryKey: ["plans", projectId] });
    },
  });
}
