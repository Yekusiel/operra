import { useState, useEffect } from "react";
import {
  CheckCircle2,
  XCircle,
  RefreshCw,
  Download,
  Terminal,
  Loader2,
  Cloud,
  AlertTriangle,
} from "lucide-react";
import { TopBar } from "../components/layout/TopBar";
import * as api from "../lib/tauri";
import type { DependencyReport, ToolStatus } from "../lib/types";

export function SetupPage() {
  const [report, setReport] = useState<DependencyReport | null>(null);
  const [checking, setChecking] = useState(false);
  const [profiles, setProfiles] = useState<string[]>([]);

  const runCheck = () => {
    setChecking(true);
    api.checkDependencies().then(setReport).finally(() => setChecking(false));
  };

  useEffect(() => {
    runCheck();
    api.listAwsProfiles().then(setProfiles);
  }, []);

  return (
    <>
      <TopBar
        title="Setup"
        subtitle="Tools, connections, and configuration"
        actions={
          <button className="btn-secondary" onClick={runCheck} disabled={checking}>
            {checking ? (
              <Loader2 className="h-4 w-4 animate-spin" />
            ) : (
              <RefreshCw className="h-4 w-4" />
            )}
            Re-check All
          </button>
        }
      />

      <div className="flex-1 p-6">
        <div className="mx-auto max-w-2xl space-y-8">
          {/* === Tools Section === */}
          <section>
            <h2 className="text-sm font-semibold text-gray-900 mb-4 uppercase tracking-wider">
              Required Tools
            </h2>

            {!report && checking && (
              <div className="flex items-center justify-center py-12">
                <div className="h-8 w-8 animate-spin rounded-full border-2 border-brand-600 border-t-transparent" />
              </div>
            )}

            {report && (
              <div className="space-y-3">
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

                {report.tools.map((tool) => (
                  <ToolCard key={tool.name} tool={tool} />
                ))}
              </div>
            )}
          </section>

          {/* === AWS Connection Section === */}
          <section>
            <h2 className="text-sm font-semibold text-gray-900 mb-4 uppercase tracking-wider">
              AWS Connection
            </h2>

            <div className="card space-y-4">
              <div className="flex items-start gap-3">
                <Cloud className="h-5 w-5 text-gray-600 mt-0.5" />
                <div className="flex-1">
                  <p className="text-sm font-medium text-gray-900">
                    AWS CLI Profiles
                  </p>
                  <p className="text-xs text-gray-500 mt-0.5">
                    Operra uses your local AWS CLI credentials to connect to AWS.
                    Each project can use a different profile.
                  </p>
                </div>
              </div>

              {profiles.length > 0 ? (
                <div>
                  <p className="text-xs font-medium text-gray-700 mb-2">
                    Available profiles:
                  </p>
                  <div className="flex flex-wrap gap-2">
                    {profiles.map((p) => (
                      <span key={p} className="badge-blue font-mono text-xs">
                        {p}
                      </span>
                    ))}
                  </div>
                </div>
              ) : (
                <div className="rounded-lg border border-yellow-200 bg-yellow-50 p-3">
                  <div className="flex items-start gap-2">
                    <AlertTriangle className="h-4 w-4 text-yellow-600 mt-0.5" />
                    <div>
                      <p className="text-sm font-medium text-yellow-800">
                        No AWS profiles found
                      </p>
                      <p className="text-xs text-yellow-700 mt-0.5">
                        Run{" "}
                        <code className="rounded bg-yellow-100 px-1 py-0.5 font-mono">
                          aws configure
                        </code>{" "}
                        to set up your credentials. You'll need an Access Key ID
                        and Secret Access Key from the AWS Console.
                      </p>
                    </div>
                  </div>
                </div>
              )}

              <div className="rounded-lg border border-gray-200 bg-gray-50 p-3">
                <p className="text-xs text-gray-600">
                  To test a specific connection, go to a project and click "Test
                  Connection" — it will validate the profile and region configured
                  for that project.
                </p>
              </div>
            </div>
          </section>
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
