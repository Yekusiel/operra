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
    description: "What level of traffic do you expect?",
    type: "select",
    options: [
      { value: "low", label: "Low (< 1K requests/day)" },
      { value: "medium", label: "Medium (1K - 100K requests/day)" },
      { value: "high", label: "High (100K - 1M requests/day)" },
      { value: "very_high", label: "Very High (1M+ requests/day)" },
      { value: "unknown", label: "Not sure yet" },
    ],
  },
  {
    key: "uptime_requirements",
    label: "Uptime Requirements",
    description: "What availability level do you need?",
    type: "select",
    options: [
      { value: "best_effort", label: "Best effort (some downtime OK)" },
      { value: "standard", label: "Standard (99.9% uptime)" },
      { value: "high", label: "High availability (99.99%)" },
      { value: "critical", label: "Mission critical (99.999%)" },
    ],
  },
  {
    key: "budget_sensitivity",
    label: "Budget Sensitivity",
    description: "How important is keeping costs low?",
    type: "select",
    options: [
      { value: "minimize", label: "Minimize cost above all" },
      { value: "balanced", label: "Balanced cost and performance" },
      { value: "performance", label: "Performance first, cost secondary" },
      { value: "no_constraint", label: "No budget constraint" },
    ],
  },
  {
    key: "database_needs",
    label: "Database Requirements",
    description: "What database capabilities do you need?",
    type: "select",
    options: [
      { value: "none", label: "No database needed" },
      { value: "relational", label: "Relational (PostgreSQL/MySQL)" },
      { value: "nosql", label: "NoSQL (DynamoDB)" },
      { value: "both", label: "Both relational and NoSQL" },
      { value: "existing", label: "Already have a database" },
    ],
  },
  {
    key: "background_jobs",
    label: "Background Jobs",
    description: "Do you need background/async processing?",
    type: "select",
    options: [
      { value: "none", label: "No background jobs" },
      { value: "simple", label: "Simple scheduled tasks" },
      { value: "queues", label: "Message queues / workers" },
      { value: "complex", label: "Complex event-driven pipelines" },
    ],
  },
  {
    key: "networking",
    label: "Networking",
    description: "What networking setup do you need?",
    type: "select",
    options: [
      { value: "public", label: "Public-facing only" },
      { value: "private", label: "Private / VPC only" },
      { value: "mixed", label: "Public frontend + private backend" },
    ],
  },
  {
    key: "storage_needs",
    label: "Storage",
    description: "Do you need file/object storage?",
    type: "select",
    options: [
      { value: "none", label: "No storage needed" },
      { value: "static", label: "Static assets only" },
      { value: "uploads", label: "User uploads" },
      { value: "large", label: "Large-scale data storage" },
    ],
  },
  {
    key: "cost_vs_performance",
    label: "Priority",
    description: "What matters most for this deployment?",
    type: "select",
    options: [
      { value: "cost", label: "Lowest possible cost" },
      { value: "speed", label: "Fastest deployment speed" },
      { value: "scalability", label: "Scalability / growth readiness" },
      { value: "simplicity", label: "Simplicity / easy maintenance" },
    ],
  },
  {
    key: "custom_notes",
    label: "Additional Notes",
    description: "Anything else the architect should know?",
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
  created_at: string;
  updated_at: string;
}

export interface PlanGenerationResult {
  plan: Plan;
  adapter_log_id: string;
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
