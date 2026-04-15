import { useParams, useNavigate, Link } from "react-router-dom";
import { useState, useEffect } from "react";
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
  Code2,
  Rocket,
  Shield,
  ChevronDown,
  ChevronRight,
  RotateCcw,
  Settings2,
  Activity,
} from "lucide-react";
import { TopBar } from "../components/layout/TopBar";
import { ScanProgressIndicator } from "../components/scanner/ScanProgress";
import { useProject, useDeleteProject } from "../hooks/useProjects";
import { useScan, useScansForProject } from "../hooks/useScan";
import { useQuestionnaire } from "../hooks/useQuestionnaire";
import { useLatestPlan, useGeneratePlan, useApprovedPlan } from "../hooks/usePlan";
import * as api from "../lib/tauri";
import type { AwsConnection } from "../lib/types";

export function ProjectDetailPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { data: project, isLoading, error } = useProject(id!);
  const deleteProject = useDeleteProject();
  const scan = useScan(id!);
  const { data: scans } = useScansForProject(id!);
  const { data: questionnaire } = useQuestionnaire(id!);
  const { data: latestPlan } = useLatestPlan(id!);

  // Consider a plan "generating" only if it started less than 5 minutes ago
  const planIsGenerating = latestPlan?.status === "generating" && (() => {
    const created = new Date(latestPlan.created_at).getTime();
    const fiveMinutes = 5 * 60 * 1000;
    return Date.now() - created < fiveMinutes;
  })();
  const planIsStuck = latestPlan?.status === "generating" && !planIsGenerating;
  const { data: approvedPlan } = useApprovedPlan(id!);
  const generatePlan = useGeneratePlan(id!);

  const [awsConn, setAwsConn] = useState<AwsConnection | null>(null);
  const [awsChecking, setAwsChecking] = useState(false);
  const [iacGenerating, setIacGenerating] = useState(false);
  const [iacResult, setIacResult] = useState<{ files: string[]; dir: string } | null>(null);
  const [iacError, setIacError] = useState<string | null>(null);
  const [tofuPlanning, setTofuPlanning] = useState(false);
  const [deployment, setDeployment] = useState<import("../lib/types").Deployment | null>(null);
  const [planError, setPlanError] = useState<string | null>(null);
  const [deployError, setDeployError] = useState<string | null>(null);
  const [applying, setApplying] = useState(false);
  const [deployKeyInfo, setDeployKeyInfo] = useState<import("../lib/types").DeployKeyInfo | null>(null);
  const [dnsInfo, setDnsInfo] = useState<import("../lib/types").DnsInstructions | null>(null);
  const [cicdSecrets, setCicdSecrets] = useState<import("../lib/types").CiCdSecrets | null>(null);
  const [copiedField, setCopiedField] = useState<string | null>(null);
  const [destroying, setDestroying] = useState(false);
  const [destroyConfirm, setDestroyConfirm] = useState(false);
  const [destroyResult, setDestroyResult] = useState<import("../lib/types").DestroyResult | null>(null);

  const [workflowOpen, setWorkflowOpen] = useState(true);

  const hasCompletedScan = scans?.some((s) => s.status === "completed");
  const hasCompletedQuestionnaire = questionnaire?.completed;
  const hasPlan = latestPlan?.status === "completed";
  const hasPlanFailed = latestPlan?.status === "failed" || planIsStuck;
  const hasPlanApproved = !!approvedPlan;
  const hasIac = !!iacResult;

  // Load persisted state on mount + poll for in-progress deployments
  useEffect(() => {
    if (!id) return;

    const loadDeployment = () => {
      api.listDeployments(id).then((deps) => {
        if (deps.length > 0) {
          setDeployment(deps[0]);
        }
      });
    };

    loadDeployment();
    api.getAwsConnection(id).then(setAwsConn);

    // Poll every 5s while a deployment is in progress
    const interval = setInterval(() => {
      if (deployment && ["planning", "approved", "applying"].includes(deployment.status)) {
        loadDeployment();
      }
    }, 5000);

    return () => clearInterval(interval);
  }, [id, deployment?.status]);

  // Restore IaC generated state from deployment history
  useEffect(() => {
    if (!project || iacResult) return;
    if (deployment && ["awaiting_approval", "approved", "completed", "failed", "planning"].includes(deployment.status)) {
      setIacResult({ files: ["(previously generated)"], dir: project.repo_path + "/infrastructure" });
    }
  }, [project, deployment]);


  const handleTestAws = () => {
    setAwsChecking(true);
    api.testAwsConnection(id!)
      .then(setAwsConn)
      .finally(() => setAwsChecking(false));
  };

  const handleGenerateIac = () => {
    if (!latestPlan) return;
    setIacGenerating(true);
    setIacError(null);
    api.generateIac(id!, latestPlan.id)
      .then((r) => {
        setIacResult({ files: r.files, dir: r.output_dir });
        if (r.deploy_key_public) {
          setDeployKeyInfo({
            public_key: r.deploy_key_public,
            github_url: `https://github.com/${project?.github_repo}/settings/keys/new`,
            instructions: "",
          });
        }
      })
      .catch((e) => setIacError(String(e)))
      .finally(() => setIacGenerating(false));
  };

  const handleTofuPlan = () => {
    setTofuPlanning(true);
    setPlanError(null);
    api.runTofuPlan(id!)
      .then(setDeployment)
      .catch((e) => setPlanError(String(e)))
      .finally(() => setTofuPlanning(false));
  };

  const handleApprove = () => {
    if (!deployment) return;
    api.approveDeployment(deployment.id).then(setDeployment);
  };

  const handleApply = () => {
    if (!deployment) return;
    setApplying(true);
    setDeployError(null);
    api.runTofuApply(deployment.id)
      .then(setDeployment)
      .catch((e) => setDeployError(String(e)))
      .finally(() => setApplying(false));
  };

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
              className="btn-secondary"
              onClick={() => navigate(`/projects/${project.id}/monitoring`)}
            >
              <Activity className="h-4 w-4" />
              Monitor
            </button>
            <button
              className="btn-secondary"
              onClick={() => navigate(`/projects/${project.id}/settings`)}
            >
              <Settings2 className="h-4 w-4" />
              Settings
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
              <dd className="mt-0.5 flex items-center gap-2 text-gray-900">
                <span>{project.aws_profile || "default"} / {project.aws_region}</span>
                {awsConn?.status === "connected" ? (
                  <span className="badge-green">Connected</span>
                ) : awsConn?.status === "failed" ? (
                  <span className="badge-red">Not connected</span>
                ) : (
                  <button
                    className="text-[10px] text-brand-600 hover:text-brand-700 underline"
                    onClick={handleTestAws}
                    disabled={awsChecking}
                  >
                    {awsChecking ? "Checking..." : "Test"}
                  </button>
                )}
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

        {/* Workflow */}
        <div className="card">
          <button
            className="flex w-full items-center justify-between"
            onClick={() => setWorkflowOpen(!workflowOpen)}
          >
            <h2 className="text-sm font-semibold text-gray-900">
              Infrastructure Workflow
            </h2>
            {workflowOpen ? (
              <ChevronDown className="h-4 w-4 text-gray-400" />
            ) : (
              <ChevronRight className="h-4 w-4 text-gray-400" />
            )}
          </button>

          {workflowOpen && (
            <div className="mt-4 space-y-3">
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

              {/* Scan progress + errors */}
              {scan.isPending && <ScanProgressIndicator progress={scan.progress} />}
              {scan.error && (
                <div className="ml-12 rounded-lg border border-red-200 bg-red-50 p-3 text-sm text-red-700">
                  Scan failed: {String(scan.error)}
                </div>
              )}

              {/* Scan history (nested under step 1) */}
              {scans && scans.length > 0 && (
                <div className="ml-12 space-y-1.5">
                  {scans.map((s) => (
                    <Link
                      key={s.id}
                      to={`/projects/${project.id}/scans/${s.id}`}
                      className="flex items-center justify-between rounded-lg border border-gray-100 bg-gray-50 px-3 py-2 text-xs transition-colors hover:bg-gray-100"
                    >
                      <div className="flex items-center gap-2">
                        <StatusIcon status={s.status} />
                        <span className="font-medium text-gray-700">
                          {s.id.slice(0, 8)}
                        </span>
                        <span className="text-gray-400">
                          {s.started_at
                            ? new Date(s.started_at).toLocaleDateString()
                            : ""}
                        </span>
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
              )}

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

              {/* Step 3: Generate & Approve Plan */}
              <WorkflowStep
                step={3}
                title={
                  hasPlanApproved
                    ? "Plan Approved"
                    : hasPlan
                      ? "Review & Approve Plan"
                      : planIsGenerating
                        ? "Generating Plan..."
                        : hasPlanFailed
                          ? "Plan Generation Failed"
                          : "Generate Infrastructure Plan"
                }
                description={
                  hasPlanApproved
                    ? "Plan approved and ready for code generation"
                    : planIsGenerating
                      ? "The AI is working on your plan. You can navigate away safely."
                      : hasPlanFailed
                        ? planIsStuck
                          ? "The previous generation appears to have stalled. Try regenerating."
                          : latestPlan?.error_msg || "Something went wrong. Try regenerating."
                        : hasPlan
                          ? "Review the plan, chat with the AI, then approve it"
                          : "Use AI to create a tailored AWS architecture plan"
                }
                status={
                  hasPlanApproved
                    ? "completed"
                    : hasPlan || planIsGenerating
                      ? "active"
                      : generatePlan.isPending
                        ? "active"
                        : hasPlanFailed
                          ? "pending"
                          : "pending"
                }
                disabled={!hasCompletedScan}
                action={
                  <div className="flex items-center gap-2">
                    {planIsGenerating ? (
                      <span className="flex items-center gap-2 text-xs text-brand-700 px-3 py-1.5">
                        <Loader2 className="h-3.5 w-3.5 animate-spin" /> Generating...
                      </span>
                    ) : hasPlanFailed ? (
                      <button
                        className="btn-primary text-xs px-3 py-1.5"
                        onClick={handleGeneratePlan}
                        disabled={generatePlan.isPending}
                      >
                        {generatePlan.isPending ? (
                          <><Loader2 className="h-3.5 w-3.5 animate-spin" /> Generating...</>
                        ) : (
                          <><RotateCcw className="h-3.5 w-3.5" /> Retry</>
                        )}
                      </button>
                    ) : hasPlan ? (
                      <Link
                        to={`/projects/${project.id}/plans/${(hasPlanApproved ? approvedPlan! : latestPlan!).id}`}
                        className="btn-primary text-xs px-3 py-1.5 no-underline"
                      >
                        <FileText className="h-3.5 w-3.5" />
                        {hasPlanApproved ? "View Plan" : "Review & Approve"}
                      </Link>
                    ) : (
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
                    )}
                    {hasPlan && (
                      <button
                        className="btn-secondary text-xs px-3 py-1.5"
                        onClick={() => {
                          if (window.confirm("Regenerate the plan? This will start a new planning session.")) {
                            generatePlan.mutate(undefined, {
                              onSuccess: (result) => navigate(`/projects/${project.id}/plans/${result.plan.id}`),
                            });
                          }
                        }}
                        disabled={generatePlan.isPending}
                      >
                        {generatePlan.isPending ? (
                          <Loader2 className="h-3.5 w-3.5 animate-spin" />
                        ) : (
                          <><RotateCcw className="h-3.5 w-3.5" /> Regenerate</>
                        )}
                      </button>
                    )}
                  </div>
                }
              />

              {generatePlan.error && (
                <div className="ml-12 rounded-lg border border-red-200 bg-red-50 p-3 text-sm text-red-700">
                  Plan generation failed: {String(generatePlan.error)}
                </div>
              )}

              {/* Step 4: Generate IaC */}
              <WorkflowStep
                step={4}
                title="Generate Infrastructure Code"
                description={hasPlanApproved ? "Create OpenTofu files from the approved plan" : "Approve a plan first to unlock this step"}
                status={hasIac ? "completed" : iacGenerating ? "active" : "pending"}
                disabled={!hasPlanApproved}
                action={
                  <button
                    className="btn-primary text-xs px-3 py-1.5"
                    onClick={handleGenerateIac}
                    disabled={!hasPlanApproved || iacGenerating}
                  >
                    {iacGenerating ? (
                      <><Loader2 className="h-3.5 w-3.5 animate-spin" /> Generating...</>
                    ) : (
                      <><Code2 className="h-3.5 w-3.5" /> Generate Code</>
                    )}
                  </button>
                }
              />

              {iacError && (
                <div className="ml-12 rounded-lg border border-red-200 bg-red-50 p-3 text-sm text-red-700">
                  IaC generation failed: {iacError}
                </div>
              )}

              {/* IaC Result inline */}
              {iacResult && (
                <div className="ml-12 rounded-lg border border-green-200 bg-green-50 p-3">
                  <p className="text-xs text-green-700 font-mono mb-1.5">{iacResult.dir}</p>
                  <div className="flex flex-wrap gap-1.5">
                    {iacResult.files.map((f) => (
                      <span key={f} className="badge-green font-mono text-[10px]">{f}</span>
                    ))}
                  </div>
                </div>
              )}

              {/* Deploy Key (shown automatically after IaC generation for GitHub projects) */}
              {deployKeyInfo && (
                <div className="ml-12 rounded-lg border border-orange-200 bg-orange-50 p-4">
                  <h4 className="text-xs font-semibold text-orange-900 mb-1">
                    GitHub Deploy Key Setup
                  </h4>
                  <p className="text-[10px] text-orange-700 mb-3">
                    Add this key to your GitHub repo before deploying so the server can clone your code.
                  </p>
                  <a
                    href={deployKeyInfo.github_url}
                    target="_blank"
                    rel="noreferrer"
                    className="text-xs text-orange-700 underline hover:text-orange-900 block mb-2"
                  >
                    Add deploy key on GitHub
                  </a>
                  <div className="flex items-start gap-2 rounded bg-white border border-orange-100 px-3 py-2">
                    <pre className="text-[10px] text-gray-700 font-mono flex-1 overflow-x-auto whitespace-pre-wrap break-all">
                      {deployKeyInfo.public_key}
                    </pre>
                    <button
                      className="text-[10px] text-orange-600 hover:text-orange-800 font-medium shrink-0"
                      onClick={() => {
                        navigator.clipboard.writeText(deployKeyInfo.public_key);
                        setCopiedField("deploy_key");
                        setTimeout(() => setCopiedField(null), 2000);
                      }}
                    >
                      {copiedField === "deploy_key" ? "Copied!" : "Copy"}
                    </button>
                  </div>
                  <p className="text-[10px] text-orange-600 mt-2">
                    Title it "Operra Deploy Key" on GitHub. Read-only access is sufficient.
                  </p>
                </div>
              )}

              {/* Step 5: Review & Approve Deployment */}
              <WorkflowStep
                step={5}
                title="Review Deployment Plan"
                description="Run tofu plan and review what will be created, updated, or destroyed"
                status={
                  deployment?.status === "awaiting_approval" || deployment?.status === "approved"
                    ? "completed"
                    : tofuPlanning
                      ? "active"
                      : "pending"
                }
                disabled={!hasIac}
                action={
                  <button
                    className="btn-primary text-xs px-3 py-1.5"
                    onClick={handleTofuPlan}
                    disabled={!hasIac || tofuPlanning}
                  >
                    {tofuPlanning ? (
                      <><Loader2 className="h-3.5 w-3.5 animate-spin" /> Planning...</>
                    ) : (
                      <><Shield className="h-3.5 w-3.5" /> Review Plan</>
                    )}
                  </button>
                }
              />

              {planError && (
                <div className="ml-12 rounded-lg border border-red-200 bg-red-50 p-3 text-sm text-red-700">
                  Plan review failed: {planError}
                </div>
              )}

              {/* Tofu plan failed (not yet approved = failed at review stage) */}
              {deployment && deployment.status === "failed" && !deployment.approved && (
                <div className="ml-12 rounded-lg border border-red-200 bg-red-50 p-3">
                  <p className="text-xs font-medium text-red-800">Plan review failed</p>
                  <pre className="mt-1.5 rounded bg-gray-900 p-2 text-[10px] text-red-400 overflow-x-auto max-h-[300px] overflow-y-auto">
                    {deployment.error_msg}
                  </pre>
                </div>
              )}

              {/* Deployment review inline */}
              {deployment && deployment.status === "awaiting_approval" && (
                <div className="ml-12 rounded-lg border border-yellow-200 bg-yellow-50 p-3">
                  <div className="flex items-center justify-between mb-2">
                    <div className="flex items-center gap-2">
                      <span className="text-xs font-medium text-yellow-800">{deployment.plan_summary}</span>
                      <span className={
                        deployment.risk_level === "high" ? "badge-red" :
                        deployment.risk_level === "medium" ? "badge-yellow" : "badge-green"
                      }>
                        {deployment.risk_level} risk
                      </span>
                    </div>
                    <button className="btn-primary text-xs px-3 py-1.5" onClick={handleApprove}>
                      <CheckCircle2 className="h-3.5 w-3.5" /> Approve
                    </button>
                  </div>
                  {deployment.plan_output && (
                    <details>
                      <summary className="text-[10px] text-yellow-700 cursor-pointer hover:text-yellow-900">
                        Full plan output
                      </summary>
                      <pre className="mt-2 rounded bg-gray-900 p-2 text-[10px] text-green-400 overflow-x-auto max-h-[300px] overflow-y-auto">
                        {deployment.plan_output}
                      </pre>
                    </details>
                  )}
                </div>
              )}

              {/* Step 6: Deploy */}
              <WorkflowStep
                step={6}
                title="Deploy to AWS"
                description="Apply the approved infrastructure changes"
                status={
                  deployment?.status === "completed"
                    ? "completed"
                    : applying
                      ? "active"
                      : "pending"
                }
                disabled={!deployment?.approved}
                action={
                  <button
                    className="btn-primary text-xs px-3 py-1.5"
                    onClick={handleApply}
                    disabled={!deployment?.approved || applying}
                  >
                    {applying ? (
                      <><Loader2 className="h-3.5 w-3.5 animate-spin" /> Deploying...</>
                    ) : (
                      <><Rocket className="h-3.5 w-3.5" /> Deploy</>
                    )}
                  </button>
                }
              />

              {deployError && (
                <div className="ml-12 rounded-lg border border-red-200 bg-red-50 p-3 text-sm text-red-700">
                  Deployment error: {deployError}
                </div>
              )}

              {/* Deploy result inline */}
              {deployment && deployment.status === "completed" && (
                <div className="ml-12 space-y-3">
                  <div className="rounded-lg border border-green-200 bg-green-50 p-3">
                    <p className="text-xs font-medium text-green-800">Deployment successful</p>
                    <p className="text-[10px] text-green-700">
                      {deployment.completed_at ? new Date(deployment.completed_at).toLocaleString() : ""}
                    </p>
                    {deployment.apply_output && (
                      <details className="mt-1.5">
                        <summary className="text-[10px] text-green-700 cursor-pointer">Apply output</summary>
                        <pre className="mt-1.5 rounded bg-gray-900 p-2 text-[10px] text-green-400 overflow-x-auto max-h-[300px] overflow-y-auto">
                          {deployment.apply_output}
                        </pre>
                      </details>
                    )}
                  </div>

                  {/* DNS Instructions */}
                  {!dnsInfo && project.domain && (
                    <button
                      className="btn-secondary text-xs w-full justify-center"
                      onClick={() => api.getDnsInstructions(id!).then(setDnsInfo)}
                    >
                      Show DNS Setup Instructions
                    </button>
                  )}
                  {dnsInfo && (
                    <div className="rounded-lg border border-blue-200 bg-blue-50 p-4">
                      <h4 className="text-xs font-semibold text-blue-900 mb-2">
                        DNS Setup for {dnsInfo.domain}
                      </h4>
                      <div className="rounded bg-white border border-blue-100 p-3 mb-3">
                        <div className="grid grid-cols-3 gap-2 text-xs">
                          <div>
                            <p className="text-gray-500">Type</p>
                            <p className="font-mono font-semibold">{dnsInfo.record_type}</p>
                          </div>
                          <div>
                            <p className="text-gray-500">Name</p>
                            <p className="font-mono font-semibold">{dnsInfo.record_name}</p>
                          </div>
                          <div>
                            <p className="text-gray-500">Value</p>
                            <p className="font-mono font-semibold">{dnsInfo.record_value}</p>
                          </div>
                        </div>
                      </div>
                      <p className="text-xs text-blue-800 whitespace-pre-line">
                        {dnsInfo.instructions}
                      </p>
                    </div>
                  )}

                  {/* CI/CD Setup */}
                  {project.source_type === "github" && !cicdSecrets && (
                    <button
                      className="btn-secondary text-xs w-full justify-center"
                      onClick={() => api.getCicdSecrets(id!).then(setCicdSecrets)}
                    >
                      Show CI/CD Setup Instructions
                    </button>
                  )}
                  {cicdSecrets && (
                    <div className="rounded-lg border border-purple-200 bg-purple-50 p-4">
                      <h4 className="text-xs font-semibold text-purple-900 mb-1">
                        CI/CD Setup (one-time)
                      </h4>
                      <p className="text-[10px] text-purple-700 mb-3">
                        Add these as <span className="font-semibold">Repository secrets</span> (not Environment secrets) so every push to <span className="font-mono font-semibold">{cicdSecrets.branch}</span> auto-deploys.
                      </p>

                      <a
                        href={cicdSecrets.secrets_url + "/new"}
                        target="_blank"
                        rel="noreferrer"
                        className="text-xs text-purple-700 underline hover:text-purple-900 block mb-3"
                      >
                        Add a new repository secret on GitHub
                      </a>

                      <div className="space-y-2">
                        <SecretRow
                          label="SERVER_IP"
                          value={cicdSecrets.server_ip}
                          copied={copiedField === "SERVER_IP"}
                          onCopy={() => { navigator.clipboard.writeText(cicdSecrets.server_ip); setCopiedField("SERVER_IP"); setTimeout(() => setCopiedField(null), 2000); }}
                        />
                        <SecretRow
                          label="SSH_USER"
                          value={cicdSecrets.ssh_user}
                          copied={copiedField === "SSH_USER"}
                          onCopy={() => { navigator.clipboard.writeText(cicdSecrets.ssh_user); setCopiedField("SSH_USER"); setTimeout(() => setCopiedField(null), 2000); }}
                        />
                        <SecretRow
                          label="SSH_PRIVATE_KEY"
                          value={cicdSecrets.ssh_private_key.length > 50 ? cicdSecrets.ssh_private_key.slice(0, 40) + "..." : cicdSecrets.ssh_private_key}
                          fullValue={cicdSecrets.ssh_private_key}
                          copied={copiedField === "SSH_PRIVATE_KEY"}
                          onCopy={() => { navigator.clipboard.writeText(cicdSecrets.ssh_private_key); setCopiedField("SSH_PRIVATE_KEY"); setTimeout(() => setCopiedField(null), 2000); }}
                        />
                      </div>

                      <p className="text-[10px] text-purple-600 mt-3">
                        After adding these secrets, push to <span className="font-mono">{cicdSecrets.branch}</span> to trigger auto-deploy.
                      </p>
                    </div>
                  )}
                </div>
              )}

              {deployment && deployment.status === "failed" && deployment.approved && (
                <div className="ml-12 rounded-lg border border-red-200 bg-red-50 p-3">
                  <p className="text-xs font-medium text-red-800">Deployment failed</p>
                  <pre className="mt-1.5 rounded bg-gray-900 p-2 text-[10px] text-red-400 overflow-x-auto max-h-[300px] overflow-y-auto">
                    {deployment.error_msg}
                  </pre>
                </div>
              )}

              {/* Destroy Infrastructure */}
              {(deployment?.status === "completed" || deployment?.status === "failed") && (
                <div className="mt-4 pt-4 border-t border-gray-200">
                  {!destroyConfirm ? (
                    <button
                      className="btn-danger w-full justify-center py-2.5 text-xs"
                      onClick={() => setDestroyConfirm(true)}
                      disabled={destroying}
                    >
                      <Trash2 className="h-3.5 w-3.5" /> Destroy Infrastructure
                    </button>
                  ) : (
                    <div className="rounded-lg border-2 border-red-300 bg-red-50 p-4">
                      <p className="text-sm font-semibold text-red-900 mb-1">
                        Are you sure?
                      </p>
                      <p className="text-xs text-red-700 mb-3">
                        This will terminate servers, delete all AWS resources, and cannot be undone.
                      </p>
                      <div className="flex items-center gap-2">
                        <button
                          className="btn-danger text-xs px-4 py-2"
                          onClick={() => {
                            setDestroying(true);
                            setDestroyConfirm(false);
                            setDestroyResult(null);
                            api.destroyInfrastructure(id!)
                              .then((result) => {
                                setDestroyResult(result);
                                if (result.success) {
                                  setDeployment(null);
                                  setIacResult(null);
                                }
                              })
                              .catch((e) => setDestroyResult({ success: false, output: String(e) }))
                              .finally(() => setDestroying(false));
                          }}
                          disabled={destroying}
                        >
                          {destroying ? (
                            <><Loader2 className="h-3.5 w-3.5 animate-spin" /> Destroying...</>
                          ) : (
                            "Yes, destroy everything"
                          )}
                        </button>
                        <button
                          className="btn-secondary text-xs px-4 py-2"
                          onClick={() => setDestroyConfirm(false)}
                        >
                          Cancel
                        </button>
                      </div>
                    </div>
                  )}

                  {destroyResult && (
                    <div className={`mt-3 rounded-lg border p-3 ${destroyResult.success ? "border-green-200 bg-green-50" : "border-red-200 bg-red-50"}`}>
                      <p className={`text-xs font-medium ${destroyResult.success ? "text-green-800" : "text-red-800"}`}>
                        {destroyResult.success ? "Infrastructure destroyed successfully" : "Destroy failed"}
                      </p>
                      <details className="mt-1.5">
                        <summary className={`text-[10px] cursor-pointer ${destroyResult.success ? "text-green-700" : "text-red-700"}`}>
                          Output
                        </summary>
                        <pre className="mt-1.5 rounded bg-gray-900 p-2 text-[10px] text-green-400 overflow-x-auto max-h-[300px] overflow-y-auto">
                          {destroyResult.output}
                        </pre>
                      </details>
                    </div>
                  )}
                </div>
              )}
            </div>
          )}
        </div>
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

function SecretRow({
  label,
  value,
  fullValue,
  copied,
  onCopy,
}: {
  label: string;
  value: string;
  fullValue?: string;
  copied: boolean;
  onCopy: () => void;
}) {
  return (
    <div className="flex items-center gap-2 rounded bg-white border border-purple-100 px-3 py-2">
      <span className="text-[10px] font-semibold text-purple-800 w-32 shrink-0 font-mono">
        {label}
      </span>
      <span className="text-[10px] text-gray-600 font-mono truncate flex-1" title={fullValue || value}>
        {value}
      </span>
      <button
        className="text-[10px] text-purple-600 hover:text-purple-800 font-medium shrink-0"
        onClick={onCopy}
      >
        {copied ? "Copied!" : "Copy"}
      </button>
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
