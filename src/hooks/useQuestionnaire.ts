import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import * as api from "../lib/tauri";

export function useQuestionnaire(projectId: string) {
  return useQuery({
    queryKey: ["questionnaire", projectId],
    queryFn: () => api.getQuestionnaire(projectId),
    enabled: !!projectId,
  });
}

export function useGetOrCreateQuestionnaire(projectId: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: () => api.getOrCreateQuestionnaire(projectId),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["questionnaire", projectId] });
    },
  });
}

export function useSaveQuestionnaire(projectId: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({
      id,
      answersJson,
      completed,
    }: {
      id: string;
      answersJson: string;
      completed: boolean;
    }) => api.saveQuestionnaire(id, answersJson, completed),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["questionnaire", projectId] });
    },
  });
}
