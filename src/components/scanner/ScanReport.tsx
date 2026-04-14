import {
  Code2,
  Layers,
  Server,
  Settings2,
  GitBranch,
  FileText,
} from "lucide-react";
import type { ScanFinding } from "../../lib/types";
import { CATEGORY_LABELS } from "../../lib/types";

interface ScanReportProps {
  findings: ScanFinding[];
  inferredStack?: string | null;
  filesScanned?: number;
  durationMs?: number;
}

const categoryIcons: Record<string, React.ElementType> = {
  language: Code2,
  framework: Layers,
  infrastructure: Server,
  config: Settings2,
  ci_cd: GitBranch,
};

const categoryBadgeClass: Record<string, string> = {
  language: "badge-blue",
  framework: "badge-purple",
  infrastructure: "badge-green",
  config: "badge-yellow",
  ci_cd: "badge-gray",
};

export function ScanReportView({
  findings,
  inferredStack,
  filesScanned,
  durationMs,
}: ScanReportProps) {
  const grouped = groupByCategory(findings);
  const categories = Object.keys(grouped);

  return (
    <div className="space-y-6">
      {/* Summary bar */}
      <div className="flex flex-wrap items-center gap-4 text-sm text-gray-600">
        {inferredStack && (
          <div className="flex items-center gap-2">
            <Layers className="h-4 w-4 text-brand-600" />
            <span className="font-medium text-gray-900">{inferredStack}</span>
          </div>
        )}
        {filesScanned !== undefined && (
          <span className="flex items-center gap-1">
            <FileText className="h-3.5 w-3.5" />
            {filesScanned.toLocaleString()} files scanned
          </span>
        )}
        {durationMs !== undefined && (
          <span>{(durationMs / 1000).toFixed(1)}s</span>
        )}
        <span>{findings.length} detections</span>
      </div>

      {/* Findings by category */}
      {categories.length === 0 && (
        <div className="rounded-lg border border-gray-200 bg-gray-50 p-8 text-center">
          <p className="text-sm text-gray-500">
            No technologies detected. This might be an empty or unfamiliar
            project structure.
          </p>
        </div>
      )}

      {categories.map((category) => {
        const Icon = categoryIcons[category] || Code2;
        const items = grouped[category];

        return (
          <div key={category} className="card">
            <div className="flex items-center gap-2 mb-4">
              <Icon className="h-4.5 w-4.5 text-gray-600" />
              <h3 className="text-sm font-semibold text-gray-900">
                {CATEGORY_LABELS[category] || category}
              </h3>
              <span className="badge-gray ml-auto">{items.length}</span>
            </div>

            <div className="space-y-3">
              {items.map((finding) => (
                <div
                  key={finding.id}
                  className="flex items-center justify-between rounded-lg border border-gray-100 bg-gray-50 px-4 py-3"
                >
                  <div className="flex items-center gap-3">
                    <span
                      className={categoryBadgeClass[category] || "badge-gray"}
                    >
                      {finding.name}
                    </span>
                  </div>

                  <div className="flex items-center gap-4 text-xs text-gray-500">
                    {finding.evidence_path && (
                      <span className="font-mono">{finding.evidence_path}</span>
                    )}
                    <span>
                      {Math.round(finding.confidence * 100)}% confidence
                    </span>
                  </div>
                </div>
              ))}
            </div>
          </div>
        );
      })}
    </div>
  );
}

function groupByCategory(
  findings: ScanFinding[]
): Record<string, ScanFinding[]> {
  const order = ["language", "framework", "infrastructure", "config", "ci_cd"];
  const grouped: Record<string, ScanFinding[]> = {};

  for (const finding of findings) {
    if (!grouped[finding.category]) {
      grouped[finding.category] = [];
    }
    grouped[finding.category].push(finding);
  }

  const result: Record<string, ScanFinding[]> = {};
  for (const cat of order) {
    if (grouped[cat]) {
      result[cat] = grouped[cat];
    }
  }
  return result;
}
