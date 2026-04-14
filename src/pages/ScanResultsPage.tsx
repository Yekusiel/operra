import { useParams, useNavigate } from "react-router-dom";
import { ArrowLeft } from "lucide-react";
import { TopBar } from "../components/layout/TopBar";
import { ScanReportView } from "../components/scanner/ScanReport";
import { useScanResults } from "../hooks/useScan";
import { useProject } from "../hooks/useProjects";

export function ScanResultsPage() {
  const { projectId, scanId } = useParams<{
    projectId: string;
    scanId: string;
  }>();
  const navigate = useNavigate();
  const { data: project } = useProject(projectId!);
  const { data, isLoading, error } = useScanResults(scanId!);

  if (isLoading) {
    return (
      <div className="flex flex-1 items-center justify-center">
        <div className="h-8 w-8 animate-spin rounded-full border-2 border-brand-600 border-t-transparent" />
      </div>
    );
  }

  if (error || !data) {
    return (
      <>
        <TopBar title="Scan Results" />
        <div className="p-6">
          <div className="rounded-lg border border-red-200 bg-red-50 p-4 text-sm text-red-700">
            {error ? String(error) : "Scan not found."}
          </div>
        </div>
      </>
    );
  }

  const [scan, findings] = data;

  return (
    <>
      <TopBar
        title="Scan Results"
        subtitle={project ? `${project.name} — Scan ${scan.id.slice(0, 8)}` : undefined}
        actions={
          <button
            className="btn-secondary"
            onClick={() => navigate(`/projects/${projectId}`)}
          >
            <ArrowLeft className="h-4 w-4" />
            Back to Project
          </button>
        }
      />

      <div className="flex-1 p-6">
        {scan.status === "failed" && (
          <div className="mb-6 rounded-lg border border-red-200 bg-red-50 p-4 text-sm text-red-700">
            Scan failed: {scan.error_msg || "Unknown error"}
          </div>
        )}

        <ScanReportView findings={findings} />
      </div>
    </>
  );
}
