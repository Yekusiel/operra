import { useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import {
  ArrowLeft,
  RefreshCw,
  Server,
  Heart,
  Activity,
  Container,
  DollarSign,
  CheckCircle2,
  XCircle,
  Clock,
  Loader2,
  Pause,
  Play,
} from "lucide-react";
import { TopBar } from "../components/layout/TopBar";
import { useProject } from "../hooks/useProjects";
import {
  useInstanceStatus,
  useAppHealth,
  useCloudWatchMetrics,
  useContainerStatus,
  useCostSummary,
} from "../hooks/useMonitoring";
import { useQueryClient } from "@tanstack/react-query";

export function MonitoringPage() {
  const { id: projectId } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { data: project } = useProject(projectId!);
  const queryClient = useQueryClient();

  const [autoRefresh, setAutoRefresh] = useState(true);
  const [metricHours, setMetricHours] = useState(1);

  const instance = useInstanceStatus(projectId!, autoRefresh);
  const health = useAppHealth(projectId!, autoRefresh);
  const cpuMetrics = useCloudWatchMetrics(projectId!, "CPUUtilization", metricHours, autoRefresh);
  const networkIn = useCloudWatchMetrics(projectId!, "NetworkIn", metricHours, autoRefresh);
  const containers = useContainerStatus(projectId!, autoRefresh);
  const cost = useCostSummary(projectId!, autoRefresh);

  const refreshAll = () => {
    queryClient.invalidateQueries({ queryKey: ["monitoring"] });
  };

  return (
    <>
      <TopBar
        title="Monitoring"
        subtitle={project?.name}
        actions={
          <div className="flex items-center gap-2">
            <button
              className={`btn-secondary text-xs px-3 py-1.5 ${autoRefresh ? "" : "opacity-60"}`}
              onClick={() => setAutoRefresh(!autoRefresh)}
            >
              {autoRefresh ? <Pause className="h-3.5 w-3.5" /> : <Play className="h-3.5 w-3.5" />}
              {autoRefresh ? "Pause" : "Resume"}
            </button>
            <button className="btn-secondary text-xs px-3 py-1.5" onClick={refreshAll}>
              <RefreshCw className="h-3.5 w-3.5" />
              Refresh
            </button>
            <button
              className="btn-secondary"
              onClick={() => navigate(`/projects/${projectId}`)}
            >
              <ArrowLeft className="h-4 w-4" />
              Back
            </button>
          </div>
        }
      />

      <div className="flex-1 overflow-y-auto p-6">
        <div className="mx-auto max-w-4xl space-y-6">
          {/* Instance Status */}
          <section>
            <SectionHeader icon={Server} title="Instance" />
            <div className="card">
              {instance.isLoading && <LoadingState />}
              {instance.error && <ErrorState error={String(instance.error)} />}
              {instance.data && (
                <div className="grid grid-cols-2 gap-4 text-sm">
                  <div>
                    <p className="text-gray-500 text-xs">Status</p>
                    <span className={
                      instance.data.state === "running" ? "badge-green" :
                      instance.data.state === "stopped" ? "badge-red" : "badge-yellow"
                    }>
                      {instance.data.state}
                    </span>
                  </div>
                  <div>
                    <p className="text-gray-500 text-xs">Public IP</p>
                    <p className="font-mono">{instance.data.public_ip || "—"}</p>
                  </div>
                  <div>
                    <p className="text-gray-500 text-xs">Instance Type</p>
                    <p className="font-mono">{instance.data.instance_type}</p>
                  </div>
                  <div>
                    <p className="text-gray-500 text-xs">Instance ID</p>
                    <p className="font-mono text-xs">{instance.data.instance_id}</p>
                  </div>
                  <div>
                    <p className="text-gray-500 text-xs">Availability Zone</p>
                    <p>{instance.data.availability_zone}</p>
                  </div>
                  <div>
                    <p className="text-gray-500 text-xs">Launch Time</p>
                    <p>{new Date(instance.data.launch_time).toLocaleString()}</p>
                  </div>
                </div>
              )}
            </div>
          </section>

          {/* App Health */}
          <section>
            <SectionHeader icon={Heart} title="App Health" />
            <div className="card">
              {health.isLoading && <LoadingState />}
              {health.error && <ErrorState error={String(health.error)} />}
              {health.data && (
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-3">
                    {health.data.healthy ? (
                      <CheckCircle2 className="h-8 w-8 text-green-500" />
                    ) : (
                      <XCircle className="h-8 w-8 text-red-500" />
                    )}
                    <div>
                      <p className="text-sm font-semibold">
                        {health.data.healthy ? "Healthy" : "Unhealthy"}
                      </p>
                      <p className="text-xs text-gray-500">
                        HTTP {health.data.status_code} &middot; {health.data.response_ms}ms
                      </p>
                    </div>
                  </div>
                  <div className="text-right text-xs text-gray-500">
                    <p className="font-mono">{health.data.url}</p>
                    <p className="flex items-center gap-1 justify-end mt-0.5">
                      <Clock className="h-3 w-3" />
                      {new Date(health.data.checked_at).toLocaleTimeString()}
                    </p>
                  </div>
                </div>
              )}
            </div>
          </section>

          {/* CPU & Network Metrics */}
          <section>
            <div className="flex items-center justify-between mb-3">
              <SectionHeader icon={Activity} title="Metrics" />
              <div className="flex gap-1">
                {[1, 6, 24].map((h) => (
                  <button
                    key={h}
                    className={`text-[10px] px-2 py-1 rounded ${
                      metricHours === h
                        ? "bg-brand-600 text-white"
                        : "bg-gray-100 text-gray-600 hover:bg-gray-200"
                    }`}
                    onClick={() => setMetricHours(h)}
                  >
                    {h}h
                  </button>
                ))}
              </div>
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div className="card">
                <p className="text-xs font-semibold text-gray-700 mb-3">CPU Utilization</p>
                {cpuMetrics.isLoading && <LoadingState />}
                {cpuMetrics.error && <ErrorState error={String(cpuMetrics.error)} />}
                {cpuMetrics.data && (
                  <MetricChart
                    datapoints={cpuMetrics.data.datapoints}
                    unit="%"
                    color="brand"
                  />
                )}
              </div>
              <div className="card">
                <p className="text-xs font-semibold text-gray-700 mb-3">Network In</p>
                {networkIn.isLoading && <LoadingState />}
                {networkIn.error && <ErrorState error={String(networkIn.error)} />}
                {networkIn.data && (
                  <MetricChart
                    datapoints={networkIn.data.datapoints}
                    unit="B"
                    color="green"
                    formatValue={formatBytes}
                  />
                )}
              </div>
            </div>
          </section>

          {/* Docker Containers */}
          <section>
            <SectionHeader icon={Container} title="Containers" />
            <div className="card">
              {containers.isLoading && <LoadingState />}
              {containers.error && <ErrorState error={String(containers.error)} />}
              {containers.data && containers.data.length === 0 && (
                <p className="text-sm text-gray-500">No containers running</p>
              )}
              {containers.data && containers.data.length > 0 && (
                <div className="space-y-2">
                  {containers.data.map((c, i) => (
                    <div
                      key={i}
                      className="flex items-center justify-between rounded-lg border border-gray-100 bg-gray-50 px-4 py-2.5"
                    >
                      <div>
                        <p className="text-sm font-medium">{c.name}</p>
                        <p className="text-[10px] text-gray-500 font-mono">{c.image}</p>
                      </div>
                      <div className="text-right">
                        <span className={
                          c.status.toLowerCase().includes("up") ? "badge-green" : "badge-red"
                        }>
                          {c.status}
                        </span>
                        {c.ports && (
                          <p className="text-[10px] text-gray-400 mt-0.5 font-mono">{c.ports}</p>
                        )}
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </section>

          {/* Cost Summary */}
          <section>
            <SectionHeader icon={DollarSign} title="Cost (Month to Date)" />
            <div className="card">
              {cost.isLoading && <LoadingState />}
              {cost.error && <ErrorState error={String(cost.error)} />}
              {cost.data && (
                <>
                  <div className="flex items-baseline gap-2 mb-4">
                    <p className="text-2xl font-bold text-gray-900">
                      ${cost.data.total}
                    </p>
                    <p className="text-sm text-gray-500">{cost.data.currency}</p>
                    <p className="text-xs text-gray-400 ml-auto">
                      {cost.data.period_start} — {cost.data.period_end}
                    </p>
                  </div>
                  {cost.data.by_service.length > 0 && (
                    <div className="space-y-1.5">
                      {cost.data.by_service.map((s, i) => (
                        <div
                          key={i}
                          className="flex items-center justify-between text-xs"
                        >
                          <span className="text-gray-600 truncate mr-4">{s.service}</span>
                          <span className="font-mono font-medium">${s.amount}</span>
                        </div>
                      ))}
                    </div>
                  )}
                </>
              )}
            </div>
          </section>

          {/* Auto-refresh indicator */}
          {autoRefresh && (
            <p className="text-center text-[10px] text-gray-400">
              Auto-refreshing every 60 seconds
            </p>
          )}
        </div>
      </div>
    </>
  );
}

function SectionHeader({ icon: Icon, title }: { icon: React.ElementType; title: string }) {
  return (
    <div className="flex items-center gap-2 mb-3">
      <Icon className="h-4 w-4 text-gray-500" />
      <h2 className="text-sm font-semibold text-gray-900 uppercase tracking-wider">{title}</h2>
    </div>
  );
}

function LoadingState() {
  return (
    <div className="flex items-center gap-2 py-4 justify-center">
      <Loader2 className="h-4 w-4 animate-spin text-gray-400" />
      <p className="text-xs text-gray-400">Loading...</p>
    </div>
  );
}

function ErrorState({ error }: { error: string }) {
  return (
    <div className="rounded-lg bg-red-50 border border-red-200 p-3">
      <p className="text-xs text-red-700">{error}</p>
    </div>
  );
}

function MetricChart({
  datapoints,
  unit,
  color,
  formatValue,
}: {
  datapoints: { timestamp: string; value: number }[];
  unit: string;
  color: "brand" | "green";
  formatValue?: (v: number) => string;
}) {
  if (datapoints.length === 0) {
    return <p className="text-xs text-gray-400 py-4 text-center">No data available</p>;
  }

  const max = Math.max(...datapoints.map((d) => d.value), 1);
  const latest = datapoints[datapoints.length - 1];
  const barColor = color === "brand" ? "bg-brand-500" : "bg-green-500";

  return (
    <div>
      <div className="flex items-end gap-[2px] h-16 mb-2">
        {datapoints.map((dp, i) => {
          const height = Math.max((dp.value / max) * 100, 2);
          return (
            <div
              key={i}
              className={`flex-1 rounded-t ${barColor} opacity-80 hover:opacity-100 transition-opacity`}
              style={{ height: `${height}%` }}
              title={`${new Date(dp.timestamp).toLocaleTimeString()}: ${
                formatValue ? formatValue(dp.value) : dp.value.toFixed(1)
              }${unit}`}
            />
          );
        })}
      </div>
      <div className="flex items-center justify-between text-[10px] text-gray-500">
        <span>{new Date(datapoints[0].timestamp).toLocaleTimeString()}</span>
        <span className="font-medium text-gray-700">
          Latest: {formatValue ? formatValue(latest.value) : latest.value.toFixed(1)}{unit}
        </span>
        <span>{new Date(latest.timestamp).toLocaleTimeString()}</span>
      </div>
    </div>
  );
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes.toFixed(0)}`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)}K`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)}M`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)}G`;
}
