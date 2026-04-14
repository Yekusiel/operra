import { useState } from "react";
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

export function useApprovedPlan(projectId: string) {
  return useQuery({
    queryKey: ["approvedPlan", projectId],
    queryFn: () => api.getApprovedPlan(projectId),
    enabled: !!projectId,
  });
}

export function useApprovePlan() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (planId: string) => api.approvePlan(planId),
    onSuccess: (plan) => {
      qc.invalidateQueries({ queryKey: ["plan", plan.id] });
      qc.invalidateQueries({ queryKey: ["latestPlan", plan.project_id] });
      qc.invalidateQueries({ queryKey: ["approvedPlan", plan.project_id] });
      qc.invalidateQueries({ queryKey: ["plans", plan.project_id] });
    },
  });
}

export function usePlanOptions(planId: string) {
  return useQuery({
    queryKey: ["planOptions", planId],
    queryFn: () => api.listPlanOptions(planId),
    enabled: !!planId,
  });
}

export function useApprovePlanOption(planId: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (optionId: string) => api.approvePlanOption(optionId),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["planOptions", planId] });
      qc.invalidateQueries({ queryKey: ["plan", planId] });
      qc.invalidateQueries({ queryKey: ["latestPlan"] });
      qc.invalidateQueries({ queryKey: ["approvedPlan"] });
    },
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

export function usePlanMessages(planId: string) {
  return useQuery({
    queryKey: ["planMessages", planId],
    queryFn: () => api.getPlanMessages(planId),
    enabled: !!planId,
  });
}

export function useSendPlanMessage(planId: string) {
  const qc = useQueryClient();
  const [isStreaming, setIsStreaming] = useState(false);

  const mutation = useMutation({
    mutationFn: (message: string) => {
      setIsStreaming(true);
      return api.sendPlanMessage(planId, message);
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["planMessages", planId] });
    },
    onSettled: () => {
      setIsStreaming(false);
    },
  });

  return { ...mutation, isStreaming };
}
