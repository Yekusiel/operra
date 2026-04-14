import { invoke, Channel } from "@tauri-apps/api/core";
import type {
  Project,
  CreateProjectInput,
  Scan,
  ScanFinding,
  ScanProgress,
  ScanReport,
  QuestionnaireResponse,
  AutoFilledAnswers,
  Plan,
  PlanGenerationResult,
} from "./types";

// ── Projects ──

export const createProject = (input: CreateProjectInput) =>
  invoke<Project>("create_project", { input });

export const listProjects = () => invoke<Project[]>("list_projects");

export const getProject = (id: string) =>
  invoke<Project | null>("get_project", { id });

export const deleteProject = (id: string) =>
  invoke<boolean>("delete_project", { id });

// ── Scanning ──

export const startScan = (
  projectId: string,
  onProgress: (progress: ScanProgress) => void
) => {
  const channel = new Channel<ScanProgress>();
  channel.onmessage = onProgress;
  return invoke<ScanReport>("start_scan", {
    projectId,
    onProgress: channel,
  });
};

export const getScanResults = (scanId: string) =>
  invoke<[Scan, ScanFinding[]]>("get_scan_results", { scanId });

export const listScansForProject = (projectId: string) =>
  invoke<Scan[]>("list_scans_for_project", { projectId });

// ── Questionnaire ──

export const getOrCreateQuestionnaire = (projectId: string) =>
  invoke<QuestionnaireResponse>("get_or_create_questionnaire", { projectId });

export const saveQuestionnaire = (
  id: string,
  answersJson: string,
  completed: boolean
) => invoke<void>("save_questionnaire", { id, answersJson, completed });

export const getQuestionnaire = (projectId: string) =>
  invoke<QuestionnaireResponse | null>("get_questionnaire", { projectId });

export const resetQuestionnaire = (projectId: string) =>
  invoke<QuestionnaireResponse>("reset_questionnaire", { projectId });

export const getAutofillSuggestions = (projectId: string) =>
  invoke<AutoFilledAnswers>("get_autofill_suggestions", { projectId });

// ── Plans ──

export const generatePlan = (projectId: string) =>
  invoke<PlanGenerationResult>("generate_plan", { projectId });

export const getPlan = (planId: string) =>
  invoke<Plan | null>("get_plan", { planId });

export const getLatestPlan = (projectId: string) =>
  invoke<Plan | null>("get_latest_plan", { projectId });

export const listPlans = (projectId: string) =>
  invoke<Plan[]>("list_plans", { projectId });
