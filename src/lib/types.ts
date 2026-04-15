export interface Project {
  id: string;
  name: string;
  source_type: "local" | "github";
  repo_path: string;
  github_repo: string | null;
  github_branch: string | null;
  aws_profile: string | null;
  aws_region: string;
  aws_access_key_id: string | null;
  aws_secret_access_key: string | null;
  domain: string | null;
  description: string | null;
  created_at: string;
  updated_at: string;
}

export interface CreateProjectInput {
  name: string;
  source_type?: string;
  repo_path?: string;
  github_repo?: string;
  github_branch?: string;
  aws_profile?: string;
  aws_region?: string;
  aws_access_key_id?: string;
  aws_secret_access_key?: string;
  domain?: string;
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
  database: "Databases",
  queue: "Queues & Background Jobs",
};

// ── Autofill ──

export interface AutoFillEntry {
  value: string;
  reason: string;
  evidence: string[];
}

export interface AutoFilledAnswers {
  database_needs: AutoFillEntry | null;
  background_jobs: AutoFillEntry | null;
  networking: AutoFillEntry | null;
  storage_needs: AutoFillEntry | null;
}

// ── Questionnaire ──

export interface QuestionnaireResponse {
  id: string;
  project_id: string;
  answers_json: string;
  completed: boolean;
  created_at: string;
  updated_at: string;
}

export interface ArchitectureAnswers {
  expected_traffic?: string;
  environment?: string;
  uptime_requirements?: string;
  preferred_region?: string;
  budget_sensitivity?: string;
  database_needs?: string;
  background_jobs?: string;
  networking?: string;
  storage_needs?: string;
  cost_vs_performance?: string;
  custom_notes?: string;
}

export interface QuestionDefinition {
  key: keyof ArchitectureAnswers;
  label: string;
  description: string;
  type: "select" | "text";
  options?: { value: string; label: string }[];
}

export const ARCHITECTURE_QUESTIONS: QuestionDefinition[] = [
  {
    key: "environment",
    label: "Environment",
    description: "What environment is this deployment for?",
    type: "select",
    options: [
      { value: "production", label: "Production" },
      { value: "staging", label: "Staging" },
      { value: "development", label: "Development" },
      { value: "both", label: "Production + Staging" },
    ],
  },
  {
    key: "expected_traffic",
    label: "Expected Traffic",
    description: "How many users or requests do you expect per day?",
    type: "select",
    options: [
      { value: "low", label: "Small — a few users or internal tool" },
      { value: "medium", label: "Moderate — hundreds to thousands of users" },
      { value: "high", label: "Large — tens of thousands of users" },
      { value: "very_high", label: "Massive — millions of requests" },
      { value: "unknown", label: "Not sure yet — let the AI decide based on my app" },
    ],
  },
  {
    key: "uptime_requirements",
    label: "Uptime Requirements",
    description: "How bad is it if your app goes down for a few minutes?",
    type: "select",
    options: [
      { value: "best_effort", label: "Not a big deal — some downtime is fine" },
      { value: "standard", label: "Should be reliable but occasional blips are OK" },
      { value: "high", label: "Needs to be up almost all the time" },
      { value: "critical", label: "Cannot go down — people or money depend on it" },
      { value: "unknown", label: "Not sure — decide based on my app type" },
    ],
  },
  {
    key: "budget_sensitivity",
    label: "Budget",
    description: "How much do you care about keeping AWS costs low?",
    type: "select",
    options: [
      { value: "minimize", label: "As cheap as possible" },
      { value: "balanced", label: "Reasonable — don't overspend but don't cripple it" },
      { value: "performance", label: "Performance matters more than cost" },
      { value: "no_constraint", label: "Cost is not a concern" },
    ],
  },
  {
    key: "database_needs",
    label: "Database",
    description:
      "Does your app need a database? If we detected one in your code, it's pre-selected.",
    type: "select",
    options: [
      { value: "none", label: "No database needed" },
      { value: "relational", label: "SQL database (PostgreSQL, MySQL)" },
      { value: "nosql", label: "NoSQL database (DynamoDB, MongoDB)" },
      { value: "both", label: "Both SQL and NoSQL" },
      { value: "existing", label: "I already have a database set up" },
      { value: "unknown", label: "Not sure — decide based on my code" },
    ],
  },
  {
    key: "background_jobs",
    label: "Background Processing",
    description:
      "Does your app need to do work in the background — like sending emails, processing uploads, or running scheduled tasks? If we detected a queue library in your code, it's pre-selected.",
    type: "select",
    options: [
      { value: "none", label: "No — everything happens in real-time requests" },
      { value: "simple", label: "Just scheduled tasks (like a daily report or cleanup)" },
      { value: "queues", label: "Yes — jobs that get queued and processed later" },
      { value: "complex", label: "Yes — complex event-driven processing" },
      { value: "unknown", label: "Not sure — decide based on my code" },
    ],
  },
  {
    key: "networking",
    label: "Access",
    description: "Who needs to reach your app?",
    type: "select",
    options: [
      { value: "public", label: "Anyone on the internet" },
      { value: "private", label: "Only internal / private network" },
      { value: "mixed", label: "Public website + private backend services" },
      { value: "unknown", label: "Not sure — decide based on my app type" },
    ],
  },
  {
    key: "storage_needs",
    label: "File Storage",
    description: "Does your app need to store files like images, documents, or downloads?",
    type: "select",
    options: [
      { value: "none", label: "No file storage needed" },
      { value: "static", label: "Just static assets (CSS, JS, images for the UI)" },
      { value: "uploads", label: "Users upload files (photos, documents, etc.)" },
      { value: "large", label: "Large amounts of data (videos, datasets, backups)" },
      { value: "unknown", label: "Not sure — decide based on my code" },
    ],
  },
  {
    key: "cost_vs_performance",
    label: "What Matters Most",
    description: "If you had to pick one priority for this deployment, what is it?",
    type: "select",
    options: [
      { value: "cost", label: "Lowest possible cost" },
      { value: "speed", label: "Ship it fast — I'll optimize later" },
      { value: "scalability", label: "Ready to grow with traffic" },
      { value: "simplicity", label: "Simple to understand and maintain" },
    ],
  },
  {
    key: "custom_notes",
    label: "Anything Else",
    description:
      "Anything the AI should know that wasn't covered above? Special requirements, existing services to integrate with, compliance needs, etc.",
    type: "text",
  },
];

// ── Plans ──

export interface Plan {
  id: string;
  project_id: string;
  scan_id: string | null;
  questionnaire_id: string | null;
  status: "pending" | "generating" | "completed" | "failed";
  plan_markdown: string | null;
  plan_json: string | null;
  alternatives: string | null;
  cost_notes: string | null;
  error_msg: string | null;
  approved: boolean;
  approved_at: string | null;
  created_at: string;
  updated_at: string;
}

export interface PlanGenerationResult {
  plan: Plan;
  adapter_log_id: string;
}

// ── Plan Options ──

export interface PlanOption {
  id: string;
  plan_id: string;
  label: string;
  title: string;
  content: string;
  source: "generation" | "chat";
  source_message_id: string | null;
  approved: boolean;
  approved_at: string | null;
  created_at: string;
}

// ── Plan Messages ──

export interface PlanMessage {
  id: string;
  plan_id: string;
  role: "user" | "assistant";
  content: string;
  created_at: string;
}

// ── Tools & Dependencies ──

export interface ToolStatus {
  name: string;
  installed: boolean;
  version: string | null;
  path: string | null;
  install_instructions: string;
  required_for: string;
}

export interface DependencyReport {
  tools: ToolStatus[];
  all_installed: boolean;
  missing_count: number;
}

// ── AWS Connection ──

export interface AwsConnection {
  id: string;
  project_id: string;
  account_id: string | null;
  arn: string | null;
  user_id: string | null;
  status: "unchecked" | "connected" | "failed";
  error_msg: string | null;
  checked_at: string | null;
  created_at: string;
}

// ── Deployments ──

export interface IacGenerationResult {
  output_dir: string;
  files: string[];
  deploy_key_public: string | null;
}

export interface Deployment {
  id: string;
  project_id: string;
  iac_id: string | null;
  action: string;
  status: string;
  plan_output: string | null;
  plan_summary: string | null;
  apply_output: string | null;
  resources_json: string | null;
  risk_level: string | null;
  approved: boolean;
  approved_at: string | null;
  error_msg: string | null;
  started_at: string | null;
  completed_at: string | null;
  created_at: string;
}

// ── Destroy ──

export interface DestroyResult {
  success: boolean;
  output: string;
}

// ── Deploy Key ──

export interface DeployKeyInfo {
  public_key: string;
  github_url: string;
  instructions: string;
}

// ── CI/CD ──

export interface CiCdSecrets {
  github_repo: string;
  secrets_url: string;
  server_ip: string;
  ssh_user: string;
  ssh_private_key: string;
  branch: string;
}

// ── DNS ──

export interface DnsInstructions {
  domain: string;
  record_type: string;
  record_name: string;
  record_value: string;
  instructions: string;
}

// ── Constants ──

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
