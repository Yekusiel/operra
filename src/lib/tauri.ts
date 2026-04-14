import { invoke, Channel } from "@tauri-apps/api/core";
import type {
  Project,
  CreateProjectInput,
  Scan,
  ScanFinding,
  ScanProgress,
  ScanReport,
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
