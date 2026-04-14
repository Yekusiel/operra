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
} from "lucide-react";
import { TopBar } from "../components/layout/TopBar";
import { ScanProgressIndicator } from "../components/scanner/ScanProgress";
import { useProject, useDeleteProject } from "../hooks/useProjects";
import { useScan, useScansForProject } from "../hooks/useScan";

export function ProjectDetailPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { data: project, isLoading, error } = useProject(id!);
  const deleteProject = useDeleteProject();
  const scan = useScan(id!);
  const { data: scans } = useScansForProject(id!);

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

        {/* Scan Action */}
        <div className="card">
          <div className="flex items-center justify-between mb-4">
            <div>
              <h2 className="text-sm font-semibold text-gray-900">
                Repository Scanner
              </h2>
              <p className="text-xs text-gray-500 mt-0.5">
                Analyze your project to detect technologies, frameworks, and
                infrastructure patterns.
              </p>
            </div>
            <button
              className="btn-primary"
              onClick={() => scan.mutate()}
              disabled={scan.isPending}
            >
              {scan.isPending ? (
                <>
                  <Loader2 className="h-4 w-4 animate-spin" />
                  Scanning...
                </>
              ) : (
                <>
                  <Scan className="h-4 w-4" />
                  Scan Repository
                </>
              )}
            </button>
          </div>

          {scan.isPending && <ScanProgressIndicator progress={scan.progress} />}

          {scan.error && (
            <div className="mt-3 rounded-lg border border-red-200 bg-red-50 p-3 text-sm text-red-700">
              Scan failed: {String(scan.error)}
            </div>
          )}

          {scan.data && !scan.isPending && (
            <div className="mt-3 rounded-lg border border-green-200 bg-green-50 p-3">
              <p className="text-sm font-medium text-green-800">
                Scan completed successfully
              </p>
              <p className="text-xs text-green-700 mt-0.5">
                Found {scan.data.detections.length} technologies across{" "}
                {scan.data.files_scanned.toLocaleString()} files
                {scan.data.inferred_stack && (
                  <> &middot; Detected: {scan.data.inferred_stack}</>
                )}
              </p>
            </div>
          )}
        </div>

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
