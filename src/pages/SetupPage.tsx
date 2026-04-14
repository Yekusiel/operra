import { useState, useEffect } from "react";
import {
  CheckCircle2,
  XCircle,
  RefreshCw,
  Download,
  Terminal,
  Loader2,
} from "lucide-react";
import { TopBar } from "../components/layout/TopBar";
import * as api from "../lib/tauri";
import type { DependencyReport, ToolStatus } from "../lib/types";

export function SetupPage() {
  const [report, setReport] = useState<DependencyReport | null>(null);
  const [checking, setChecking] = useState(false);

  const runCheck = () => {
    setChecking(true);
    api.checkDependencies().then(setReport).finally(() => setChecking(false));
  };

  useEffect(() => {
    runCheck();
  }, []);

  return (
    <>
      <TopBar
        title="Setup"
        subtitle="Required tools and dependencies"
        actions={
          <button className="btn-secondary" onClick={runCheck} disabled={checking}>
            {checking ? (
              <Loader2 className="h-4 w-4 animate-spin" />
            ) : (
              <RefreshCw className="h-4 w-4" />
            )}
            Re-check
          </button>
        }
      />

      <div className="flex-1 p-6">
        <div className="mx-auto max-w-2xl space-y-4">
          {!report && checking && (
            <div className="flex items-center justify-center py-20">
              <div className="h-8 w-8 animate-spin rounded-full border-2 border-brand-600 border-t-transparent" />
            </div>
          )}

          {report && (
            <>
              {/* Summary */}
              <div
                className={`card ${
                  report.all_installed
                    ? "border-green-200 bg-green-50/30"
                    : "border-yellow-200 bg-yellow-50/30"
                }`}
              >
                <div className="flex items-center gap-3">
                  {report.all_installed ? (
                    <CheckCircle2 className="h-6 w-6 text-green-600" />
                  ) : (
                    <Download className="h-6 w-6 text-yellow-600" />
                  )}
                  <div>
                    <p className="text-sm font-semibold text-gray-900">
                      {report.all_installed
                        ? "All tools installed"
                        : `${report.missing_count} tool${report.missing_count > 1 ? "s" : ""} missing`}
                    </p>
                    <p className="text-xs text-gray-500">
                      {report.all_installed
                        ? "Operra is fully configured and ready to use."
                        : "Install the missing tools to unlock all features."}
                    </p>
                  </div>
                </div>
              </div>

              {/* Tool cards */}
              {report.tools.map((tool) => (
                <ToolCard key={tool.name} tool={tool} />
              ))}
            </>
          )}
        </div>
      </div>
    </>
  );
}

function ToolCard({ tool }: { tool: ToolStatus }) {
  const [showInstructions, setShowInstructions] = useState(!tool.installed);

  return (
    <div
      className={`card ${
        tool.installed ? "border-green-200" : "border-red-200"
      }`}
    >
      <div className="flex items-start justify-between">
        <div className="flex items-start gap-3">
          {tool.installed ? (
            <CheckCircle2 className="h-5 w-5 text-green-600 mt-0.5" />
          ) : (
            <XCircle className="h-5 w-5 text-red-500 mt-0.5" />
          )}
          <div>
            <h3 className="text-sm font-semibold text-gray-900">
              {tool.name}
            </h3>
            <p className="text-xs text-gray-500 mt-0.5">{tool.required_for}</p>
            {tool.installed && tool.version && (
              <p className="text-xs text-green-700 mt-1 font-mono">
                {tool.version}
              </p>
            )}
            {tool.installed && tool.path && (
              <p className="text-[10px] text-gray-400 mt-0.5 font-mono truncate max-w-md">
                {tool.path}
              </p>
            )}
          </div>
        </div>
        <span className={tool.installed ? "badge-green" : "badge-red"}>
          {tool.installed ? "Installed" : "Missing"}
        </span>
      </div>

      {!tool.installed && (
        <div className="mt-4">
          <button
            className="flex items-center gap-1 text-xs text-brand-600 hover:text-brand-700"
            onClick={() => setShowInstructions(!showInstructions)}
          >
            <Terminal className="h-3.5 w-3.5" />
            {showInstructions ? "Hide" : "Show"} install instructions
          </button>

          {showInstructions && (
            <pre className="mt-2 rounded-lg bg-gray-900 p-4 text-xs text-green-400 overflow-x-auto whitespace-pre-wrap">
              {tool.install_instructions}
            </pre>
          )}
        </div>
      )}
    </div>
  );
}
