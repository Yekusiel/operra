export interface Project {
  id: string;
  name: string;
  repo_path: string;
  aws_profile: string | null;
  aws_region: string;
  description: string | null;
  created_at: string;
  updated_at: string;
}

export interface CreateProjectInput {
  name: string;
  repo_path: string;
  aws_profile?: string;
  aws_region?: string;
  description?: string;
}

export interface Scan {
  id: string;
  project_id: string;
  status: "pending" | "running" | "completed" | "failed";
  started_at: string | null;
  completed_at: string | null;
  error_msg: string | null;
  created_at: string;
}

export interface ScanFinding {
  id: string;
  scan_id: string;
  category: "language" | "framework" | "infrastructure" | "config" | "ci_cd";
  name: string;
  confidence: number;
  evidence_path: string | null;
  metadata_json: string | null;
  created_at: string;
}

export interface ScanProgress {
  phase: string;
  files_checked: number;
  detections_so_far: number;
}

export interface ScanReport {
  detections: Detection[];
  files_scanned: number;
  duration_ms: number;
  inferred_stack: string | null;
}

export interface Detection {
  category: string;
  name: string;
  confidence: number;
  evidence_path: string;
  metadata: Record<string, unknown> | null;
}

export const CATEGORY_LABELS: Record<string, string> = {
  language: "Languages",
  framework: "Frameworks",
  infrastructure: "Infrastructure",
  config: "Configuration",
  ci_cd: "CI/CD",
};

export const AWS_REGIONS = [
  { value: "us-east-1", label: "US East (N. Virginia)" },
  { value: "us-east-2", label: "US East (Ohio)" },
  { value: "us-west-1", label: "US West (N. California)" },
  { value: "us-west-2", label: "US West (Oregon)" },
  { value: "eu-west-1", label: "EU (Ireland)" },
  { value: "eu-west-2", label: "EU (London)" },
  { value: "eu-central-1", label: "EU (Frankfurt)" },
  { value: "ap-southeast-1", label: "Asia Pacific (Singapore)" },
  { value: "ap-southeast-2", label: "Asia Pacific (Sydney)" },
  { value: "ap-northeast-1", label: "Asia Pacific (Tokyo)" },
] as const;
