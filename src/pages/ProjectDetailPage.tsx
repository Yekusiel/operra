import { useParams, useNavigate, Link } from "react-router-dom";
import {
  ArrowLeft,
  Scan,
  Trash2,
  FolderGit2,
  Cloud,
  Clock,
  CheckCircle2,
  XCircle,
  Loader2,
  ClipboardList,
  Cpu,
  FileText,
} from "lucide-react";
import { TopBar } from "../components/layout/TopBar";
import { ScanProgressIndicator } from "../components/scanner/ScanProgress";
import { useProject, useDeleteProject } from "../hooks/useProjects";
import { useScan, useScansForProject } from "../hooks/useScan";
import { useQuestionnaire } from "../hooks/useQuestionnaire";
import { useLatestPlan, useGeneratePlan } from "../hooks/usePlan";

export function ProjectDetailPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { data: project, isLoading, error } = useProject(id!);
  const deleteProject = useDeleteProject();
  const scan = useScan(id!);
  const { data: scans } = useScansForProject(id!);
  const { data: questionnaire } = useQuestionnaire(id!);
  const { data: latestPlan } = useLatestPlan(id!);
  const generatePlan = useGeneratePlan(id!);

  const hasCompletedScan = scans?.some((s) => s.status === "completed");
  const hasCompletedQuestionnaire = questionnaire?.completed;

  if (isLoading) {
    return (
      <div className="flex flex-1 items-center justify-center">
        <div className="h-8 w-8 animate-spin rounded-full border-2 border-brand-600 border-t-transparent" />
      </div>
    );
  }

  if (error || !project) {
    return (
      <>
        <TopBar title="Project Not Found" />
        <div className="p-6">
          <div className="rounded-lg border border-red-200 bg-red-50 p-4 text-sm text-red-700">
            {error ? String(error) : "Project not found."}
          </div>
          <button className="btn-secondary mt-4" onClick={() => navigate("/")}>
            <ArrowLeft className="h-4 w-4" />
            Back to Projects
          </button>
        </div>
      </>
    );
  }

  const handleDelete = () => {
    if (window.confirm(`Delete project "${project.name}"? This cannot be undone.`)) {
      deleteProject.mutate(project.id, {
        onSuccess: () => navigate("/"),
      });
    }
  };

  const handleGeneratePlan = () => {
    generatePlan.mutate(undefined, {
      onSuccess: (result) => {
        navigate(`/projects/${project.id}/plans/${result.plan.id}`);
      },
    });
  };

  return (
    <>
      <TopBar
        title={project.name}
        subtitle={project.description || undefined}
        actions={
          <div className="flex items-center gap-2">
            <button className="btn-secondary" onClick={() => navigate("/")}>
              <ArrowLeft className="h-4 w-4" />
              Back
            </button>
            <button
              className="btn-danger"
              onClick={handleDelete}
              disabled={deleteProject.isPending}
            >
              <Trash2 className="h-4 w-4" />
              Delete
            </button>
          </div>
        }
      />

      <div className="flex-1 space-y-6 p-6">
        {/* Project Info */}
        <div className="card">
          <h2 className="mb-4 text-sm font-semibold text-gray-900">
            Project Details
          </h2>
          <dl className="grid grid-cols-2 gap-4 text-sm">
            <div>
              <dt className="flex items-center gap-1.5 text-gray-500">
                <FolderGit2 className="h-3.5 w-3.5" />
                Repository
              </dt>
              <dd className="mt-0.5 font-mono text-gray-900">
                {project.repo_path}
              </dd>
            </div>
            <div>
              <dt className="flex items-center gap-1.5 text-gray-500">
                <Cloud className="h-3.5 w-3.5" />
                AWS
              </dt>
              <dd className="mt-0.5 text-gray-900">
                {project.aws_profile || "Not configured"} /{" "}
                {project.aws_region}
              </dd>
            </div>
            <div>
              <dt className="flex items-center gap-1.5 text-gray-500">
                <Clock className="h-3.5 w-3.5" />
                Created
              </dt>
              <dd className="mt-0.5 text-gray-900">
                {new Date(project.created_at).toLocaleString()}
              </dd>
            </div>
          </dl>
        </div>

        {/* Workflow Steps */}
        <div className="card">
          <h2 className="mb-4 text-sm font-semibold text-gray-900">
            Infrastructure Workflow
          </h2>
          <div className="space-y-3">
            {/* Step 1: Scan */}
            <WorkflowStep
              step={1}
              title="Scan Repository"
              description="Detect technologies, frameworks, and infrastructure patterns"
              status={hasCompletedScan ? "completed" : scan.isPending ? "active" : "pending"}
              action={
                <button
                  className="btn-primary text-xs px-3 py-1.5"
                  onClick={() => scan.mutate()}
                  disabled={scan.isPending}
                >
                  {scan.isPending ? (
                    <><Loader2 className="h-3.5 w-3.5 animate-spin" /> Scanning...</>
                  ) : hasCompletedScan ? (
                    <><Scan className="h-3.5 w-3.5" /> Re-scan</>
                  ) : (
                    <><Scan className="h-3.5 w-3.5" /> Scan</>
                  )}
                </button>
              }
            />

            {/* Step 2: Questionnaire */}
            <WorkflowStep
              step={2}
              title="Architecture Questionnaire"
              description="Answer questions about your infrastructure requirements"
              status={hasCompletedQuestionnaire ? "completed" : "pending"}
              disabled={!hasCompletedScan}
              action={
                <button
                  className="btn-primary text-xs px-3 py-1.5"
                  onClick={() => navigate(`/projects/${project.id}/questionnaire`)}
                  disabled={!hasCompletedScan}
                >
                  <ClipboardList className="h-3.5 w-3.5" />
                  {hasCompletedQuestionnaire ? "Edit Answers" : "Start"}
                </button>
              }
            />

            {/* Step 3: Generate Plan */}
            <WorkflowStep
              step={3}
              title="Generate Infrastructure Plan"
              description="Use AI to create a tailored AWS architecture plan"
              status={
                latestPlan?.status === "completed"
                  ? "completed"
                  : generatePlan.isPending
                    ? "active"
                    : "pending"
              }
              disabled={!hasCompletedScan}
              action={
                <button
                  className="btn-primary text-xs px-3 py-1.5"
                  onClick={handleGeneratePlan}
                  disabled={!hasCompletedScan || generatePlan.isPending}
                >
                  {generatePlan.isPending ? (
                    <><Loader2 className="h-3.5 w-3.5 animate-spin" /> Generating...</>
                  ) : (
                    <><Cpu className="h-3.5 w-3.5" /> Generate Plan</>
                  )}
                </button>
              }
            />
          </div>
        </div>

        {/* Scan progress indicator */}
        {scan.isPending && <ScanProgressIndicator progress={scan.progress} />}

        {scan.error && (
          <div className="rounded-lg border border-red-200 bg-red-50 p-3 text-sm text-red-700">
            Scan failed: {String(scan.error)}
          </div>
        )}

        {generatePlan.error && (
          <div className="rounded-lg border border-red-200 bg-red-50 p-4 text-sm text-red-700">
            Plan generation failed: {String(generatePlan.error)}
          </div>
        )}

        {/* Latest Plan Quick View */}
        {latestPlan && latestPlan.status === "completed" && (
          <div className="card border-brand-200 bg-brand-50/30">
            <div className="flex items-center justify-between mb-2">
              <div className="flex items-center gap-2">
                <FileText className="h-5 w-5 text-brand-600" />
                <h3 className="text-sm font-semibold text-brand-900">
                  Latest Infrastructure Plan
                </h3>
              </div>
              <Link
                to={`/projects/${project.id}/plans/${latestPlan.id}`}
                className="btn-primary text-xs px-3 py-1.5"
              >
                View Full Plan
              </Link>
            </div>
            <p className="text-xs text-brand-700">
              Generated {new Date(latestPlan.created_at).toLocaleString()}
            </p>
            {latestPlan.plan_markdown && (
              <p className="text-sm text-brand-800 mt-2 line-clamp-3">
                {latestPlan.plan_markdown.slice(0, 300)}...
              </p>
            )}
          </div>
        )}

        {/* Scan History */}
        {scans && scans.length > 0 && (
          <div className="card">
            <h2 className="mb-4 text-sm font-semibold text-gray-900">
              Scan History
            </h2>
            <div className="space-y-2">
              {scans.map((s) => (
                <Link
                  key={s.id}
                  to={`/projects/${project.id}/scans/${s.id}`}
                  className="flex items-center justify-between rounded-lg border border-gray-100 bg-gray-50 px-4 py-3 transition-colors hover:bg-gray-100"
                >
                  <div className="flex items-center gap-3">
                    <StatusIcon status={s.status} />
                    <div>
                      <p className="text-sm font-medium text-gray-900">
                        Scan {s.id.slice(0, 8)}
                      </p>
                      <p className="text-xs text-gray-500">
                        {s.started_at
                          ? new Date(s.started_at).toLocaleString()
                          : s.created_at}
                      </p>
                    </div>
                  </div>
                  <span
                    className={
                      s.status === "completed"
                        ? "badge-green"
                        : s.status === "failed"
                          ? "badge-red"
                          : s.status === "running"
                            ? "badge-blue"
                            : "badge-gray"
                    }
                  >
                    {s.status}
                  </span>
                </Link>
              ))}
            </div>
          </div>
        )}
      </div>
    </>
  );
}

function WorkflowStep({
  step,
  title,
  description,
  status,
  action,
  disabled,
}: {
  step: number;
  title: string;
  description: string;
  status: "pending" | "active" | "completed";
  action: React.ReactNode;
  disabled?: boolean;
}) {
  return (
    <div
      className={`flex items-center gap-4 rounded-lg border px-4 py-3 ${
        disabled
          ? "border-gray-100 bg-gray-50 opacity-60"
          : status === "completed"
            ? "border-green-200 bg-green-50/50"
            : status === "active"
              ? "border-brand-200 bg-brand-50/50"
              : "border-gray-200 bg-white"
      }`}
    >
      <div
        className={`flex h-8 w-8 shrink-0 items-center justify-center rounded-full text-sm font-semibold ${
          status === "completed"
            ? "bg-green-100 text-green-700"
            : status === "active"
              ? "bg-brand-100 text-brand-700"
              : "bg-gray-100 text-gray-500"
        }`}
      >
        {status === "completed" ? (
          <CheckCircle2 className="h-4 w-4" />
        ) : (
          step
        )}
      </div>
      <div className="flex-1 min-w-0">
        <p className="text-sm font-medium text-gray-900">{title}</p>
        <p className="text-xs text-gray-500">{description}</p>
      </div>
      {action}
    </div>
  );
}

function StatusIcon({ status }: { status: string }) {
  switch (status) {
    case "completed":
      return <CheckCircle2 className="h-5 w-5 text-green-500" />;
    case "failed":
      return <XCircle className="h-5 w-5 text-red-500" />;
    case "running":
      return <Loader2 className="h-5 w-5 animate-spin text-brand-500" />;
    default:
      return <Clock className="h-5 w-5 text-gray-400" />;
  }
}
