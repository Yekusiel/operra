import { useParams, useNavigate } from "react-router-dom";
import {
  ArrowLeft,
  FileText,
  AlertTriangle,
  DollarSign,
  Lightbulb,
  Clock,
} from "lucide-react";
import { TopBar } from "../components/layout/TopBar";
import { useProject } from "../hooks/useProjects";
import { usePlan } from "../hooks/usePlan";

export function PlanViewPage() {
  const { projectId, planId } = useParams<{
    projectId: string;
    planId: string;
  }>();
  const navigate = useNavigate();
  const { data: project } = useProject(projectId!);
  const { data: plan, isLoading, error } = usePlan(planId!);

  if (isLoading) {
    return (
      <div className="flex flex-1 items-center justify-center">
        <div className="h-8 w-8 animate-spin rounded-full border-2 border-brand-600 border-t-transparent" />
      </div>
    );
  }

  if (error || !plan) {
    return (
      <>
        <TopBar title="Infrastructure Plan" />
        <div className="p-6">
          <div className="rounded-lg border border-red-200 bg-red-50 p-4 text-sm text-red-700">
            {error ? String(error) : "Plan not found."}
          </div>
        </div>
      </>
    );
  }

  return (
    <>
      <TopBar
        title="Infrastructure Plan"
        subtitle={project?.name}
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
        <div className="mx-auto max-w-4xl space-y-6">
          {/* Status header */}
          <div className="flex items-center gap-4">
            <span
              className={
                plan.status === "completed"
                  ? "badge-green"
                  : plan.status === "failed"
                    ? "badge-red"
                    : plan.status === "generating"
                      ? "badge-blue"
                      : "badge-gray"
              }
            >
              {plan.status}
            </span>
            <span className="flex items-center gap-1 text-xs text-gray-500">
              <Clock className="h-3.5 w-3.5" />
              {new Date(plan.created_at).toLocaleString()}
            </span>
          </div>

          {plan.status === "failed" && (
            <div className="rounded-lg border border-red-200 bg-red-50 p-4">
              <div className="flex items-start gap-2">
                <AlertTriangle className="h-5 w-5 text-red-500 mt-0.5" />
                <div>
                  <p className="text-sm font-medium text-red-800">
                    Plan generation failed
                  </p>
                  <p className="text-sm text-red-700 mt-1">
                    {plan.error_msg || "Unknown error"}
                  </p>
                </div>
              </div>
            </div>
          )}

          {/* Main plan content */}
          {plan.plan_markdown && (
            <div className="card">
              <div className="flex items-center gap-2 mb-4">
                <FileText className="h-5 w-5 text-brand-600" />
                <h2 className="text-base font-semibold text-gray-900">
                  Recommended Architecture
                </h2>
              </div>
              <div className="prose prose-sm max-w-none">
                <MarkdownRenderer content={plan.plan_markdown} />
              </div>
            </div>
          )}

          {/* Alternatives sidebar */}
          {plan.alternatives && (
            <div className="card border-purple-200 bg-purple-50/30">
              <div className="flex items-center gap-2 mb-3">
                <Lightbulb className="h-5 w-5 text-purple-600" />
                <h3 className="text-sm font-semibold text-purple-900">
                  Alternatives
                </h3>
              </div>
              <div className="prose prose-sm max-w-none text-purple-900">
                <MarkdownRenderer content={plan.alternatives} />
              </div>
            </div>
          )}

          {/* Cost notes */}
          {plan.cost_notes && (
            <div className="card border-green-200 bg-green-50/30">
              <div className="flex items-center gap-2 mb-3">
                <DollarSign className="h-5 w-5 text-green-600" />
                <h3 className="text-sm font-semibold text-green-900">
                  Cost Notes
                </h3>
              </div>
              <div className="prose prose-sm max-w-none text-green-900">
                <MarkdownRenderer content={plan.cost_notes} />
              </div>
            </div>
          )}
        </div>
      </div>
    </>
  );
}

function MarkdownRenderer({ content }: { content: string }) {
  // Simple markdown-to-HTML for plan content.
  // Handles headers, bold, italic, lists, and code blocks.
  const html = content
    .replace(/^#### (.+)$/gm, '<h4 class="text-sm font-semibold mt-4 mb-2">$1</h4>')
    .replace(/^### (.+)$/gm, '<h3 class="text-base font-semibold mt-6 mb-2">$1</h3>')
    .replace(/^## (.+)$/gm, '<h2 class="text-lg font-semibold mt-6 mb-3">$1</h2>')
    .replace(/\*\*(.+?)\*\*/g, "<strong>$1</strong>")
    .replace(/\*(.+?)\*/g, "<em>$1</em>")
    .replace(/`([^`]+)`/g, '<code class="rounded bg-gray-100 px-1.5 py-0.5 text-xs">$1</code>')
    .replace(/^- (.+)$/gm, '<li class="ml-4 list-disc">$1</li>')
    .replace(/^(\d+)\. (.+)$/gm, '<li class="ml-4 list-decimal">$2</li>')
    .replace(/\n\n/g, '</p><p class="mb-3">')
    .replace(/\n/g, "<br />");

  return (
    <div
      className="text-sm leading-relaxed text-gray-800"
      dangerouslySetInnerHTML={{ __html: `<p class="mb-3">${html}</p>` }}
    />
  );
}
