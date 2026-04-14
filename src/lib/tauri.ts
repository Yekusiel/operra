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
  PlanOption,
  PlanMessage,
  DependencyReport,
  AwsConnection,
  IacGenerationResult,
  Deployment,
  DnsInstructions,
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

export const generateAdditionalOption = (planId: string, userRequest?: string) =>
  invoke<import("./types").PlanOption>("generate_additional_option", { planId, userRequest: userRequest || null });

export const approvePlan = (planId: string) =>
  invoke<Plan>("approve_plan", { planId });

export const getApprovedPlan = (projectId: string) =>
  invoke<Plan | null>("get_approved_plan", { projectId });

export const listPlanOptions = (planId: string) =>
  invoke<PlanOption[]>("list_plan_options", { planId });

export const approvePlanOption = (optionId: string) =>
  invoke<PlanOption>("approve_plan_option", { optionId });

export const getApprovedOption = (planId: string) =>
  invoke<PlanOption | null>("get_approved_option", { planId });

// ── Plan Chat ──

export const sendPlanMessage = (planId: string, message: string) =>
  invoke<PlanMessage>("send_plan_message", { planId, message });

export const getPlanMessages = (planId: string) =>
  invoke<PlanMessage[]>("get_plan_messages", { planId });

// ── Tools & Dependencies ──

export const checkDependencies = () =>
  invoke<DependencyReport>("check_dependencies");

// ── AWS Connection ──

export const testAwsConnection = (projectId: string) =>
  invoke<AwsConnection>("test_aws_connection", { projectId });

export const getAwsConnection = (projectId: string) =>
  invoke<AwsConnection | null>("get_aws_connection", { projectId });

export const listAwsProfiles = () =>
  invoke<string[]>("list_aws_profiles");

// ── Deployment ──

export const generateIac = (projectId: string, planId: string) =>
  invoke<IacGenerationResult>("generate_iac", { projectId, planId });

export const runTofuPlan = (projectId: string) =>
  invoke<Deployment>("run_tofu_plan", { projectId });

export const approveDeployment = (deploymentId: string) =>
  invoke<Deployment>("approve_deployment", { deploymentId });

export const runTofuApply = (deploymentId: string) =>
  invoke<Deployment>("run_tofu_apply", { deploymentId });

export const getDeployment = (deploymentId: string) =>
  invoke<Deployment | null>("get_deployment", { deploymentId });

export const listDeployments = (projectId: string) =>
  invoke<Deployment[]>("list_deployments", { projectId });

// ── DNS ──

export const getDnsInstructions = (projectId: string) =>
  invoke<DnsInstructions | null>("get_dns_instructions", { projectId });
