import { useState, useRef, useEffect } from "react";
import { useParams, useNavigate } from "react-router-dom";
import {
  ArrowLeft,
  AlertTriangle,
  Clock,
  Send,
  Loader2,
  User,
  Bot,
  CheckCircle2,
  Shield,
  Plus,
} from "lucide-react";
import { TopBar } from "../components/layout/TopBar";
import { useProject } from "../hooks/useProjects";
import {
  usePlan,
  usePlanMessages,
  useSendPlanMessage,
  usePlanOptions,
  useApprovePlanOption,
  useGenerateAdditionalOption,
} from "../hooks/usePlan";

export function PlanViewPage() {
  const { projectId, planId } = useParams<{
    projectId: string;
    planId: string;
  }>();
  const navigate = useNavigate();
  const { data: project } = useProject(projectId!);
  const { data: plan, isLoading, error } = usePlan(planId!);
  const { data: options } = usePlanOptions(planId!);
  const { data: messages } = usePlanMessages(planId!);
  const sendMessage = useSendPlanMessage(planId!);
  const approveOption = useApprovePlanOption(planId!);
  const addOption = useGenerateAdditionalOption(planId!);

  const [input, setInput] = useState("");
  const chatEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    chatEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages, sendMessage.isStreaming]);

  const handleSend = () => {
    const text = input.trim();
    if (!text || sendMessage.isPending) return;
    setInput("");
    sendMessage.mutate(text);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

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
        <TopBar title="Infrastructure Planning" />
        <div className="p-6">
          <div className="rounded-lg border border-red-200 bg-red-50 p-4 text-sm text-red-700">
            {error ? String(error) : "Plan not found."}
          </div>
        </div>
      </>
    );
  }

  const hasApprovedOption = options?.some((o) => o.approved);
  const hasOptions = options && options.length > 0;

  return (
    <>
      <TopBar
        title="Infrastructure Planning"
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

      <div className="flex flex-1 flex-col overflow-hidden">
        <div className="flex-1 overflow-y-auto p-6">
          <div className="mx-auto max-w-4xl space-y-6">
            {/* Status */}
            <div className="flex items-center gap-4">
              <span
                className={
                  hasApprovedOption
                    ? "badge-green"
                    : plan.status === "completed"
                      ? "badge-yellow"
                      : plan.status === "failed"
                        ? "badge-red"
                        : "badge-blue"
                }
              >
                {hasApprovedOption
                  ? "plan approved"
                  : plan.status === "generating"
                    ? "generating options..."
                    : plan.status}
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

            {/* Instruction banner */}
            {plan.status === "completed" && !hasApprovedOption && hasOptions && (
              <div className="rounded-xl border-2 border-brand-200 bg-brand-50 p-4">
                <div className="flex items-start gap-3">
                  <Shield className="h-5 w-5 text-brand-600 mt-0.5" />
                  <div>
                    <p className="text-sm font-semibold text-brand-900">
                      Choose a plan to approve
                    </p>
                    <p className="text-sm text-brand-700 mt-0.5">
                      Review the options below, use the chat to ask questions,
                      or add another option. Approve the one you want to build.
                    </p>
                  </div>
                </div>
              </div>
            )}

            {/* Plan Option Cards */}
            {hasOptions &&
              options.map((option) => (
                <div
                  key={option.id}
                  className={`card relative ${
                    option.approved
                      ? "border-2 border-green-400 bg-green-50/30"
                      : "hover:border-gray-300"
                  }`}
                >
                  {/* Header */}
                  <div className="flex items-start justify-between gap-4 mb-4">
                    <div className="flex items-center gap-3">
                      <span
                        className={`flex h-9 w-9 items-center justify-center rounded-full text-sm font-bold ${
                          option.approved
                            ? "bg-green-600 text-white"
                            : "bg-brand-100 text-brand-700"
                        }`}
                      >
                        {option.label}
                      </span>
                      <div>
                        <h3 className="text-base font-semibold text-gray-900">
                          {option.title}
                        </h3>
                        {option.source === "chat" && (
                          <span className="text-[10px] text-gray-400 uppercase tracking-wider">
                            Added by request
                          </span>
                        )}
                      </div>
                    </div>

                    {option.approved ? (
                      <div className="flex items-center gap-2">
                        <CheckCircle2 className="h-5 w-5 text-green-600" />
                        <div className="text-right">
                          <p className="text-xs font-semibold text-green-800">
                            Approved
                          </p>
                          <p className="text-[10px] text-green-600">
                            {option.approved_at
                              ? new Date(option.approved_at).toLocaleString()
                              : ""}
                          </p>
                        </div>
                      </div>
                    ) : (
                      <button
                        className="btn-primary shrink-0"
                        onClick={() => approveOption.mutate(option.id)}
                        disabled={approveOption.isPending}
                      >
                        {approveOption.isPending ? (
                          <Loader2 className="h-4 w-4 animate-spin" />
                        ) : (
                          <CheckCircle2 className="h-4 w-4" />
                        )}
                        Approve Plan {option.label}
                      </button>
                    )}
                  </div>

                  {/* Content */}
                  <div className="prose prose-sm max-w-none">
                    <MarkdownRenderer content={option.content} />
                  </div>
                </div>
              ))}

            {/* Add Another Option button */}
            {plan.status === "completed" && (
              <button
                className="btn-secondary w-full justify-center py-3"
                onClick={() => addOption.mutate(undefined)}
                disabled={addOption.isPending}
              >
                {addOption.isPending ? (
                  <>
                    <Loader2 className="h-4 w-4 animate-spin" />
                    Generating another option...
                  </>
                ) : (
                  <>
                    <Plus className="h-4 w-4" />
                    Add Another Option
                  </>
                )}
              </button>
            )}

            {addOption.error && (
              <div className="rounded-lg border border-red-200 bg-red-50 p-3 text-sm text-red-700">
                Failed to generate option: {String(addOption.error)}
              </div>
            )}

            {/* Chat messages */}
            {messages && messages.length > 0 && (
              <div className="space-y-4 pt-2">
                <div className="flex items-center gap-2">
                  <div className="h-px flex-1 bg-gray-200" />
                  <span className="text-xs font-medium text-gray-400 uppercase tracking-wider">
                    Conversation
                  </span>
                  <div className="h-px flex-1 bg-gray-200" />
                </div>

                {messages.map((msg) => (
                  <div
                    key={msg.id}
                    className={`flex gap-3 ${
                      msg.role === "user" ? "justify-end" : "justify-start"
                    }`}
                  >
                    {msg.role === "assistant" && (
                      <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-brand-100">
                        <Bot className="h-4 w-4 text-brand-700" />
                      </div>
                    )}
                    <div
                      className={`max-w-[80%] rounded-xl px-4 py-3 ${
                        msg.role === "user"
                          ? "bg-brand-600 text-white"
                          : "card"
                      }`}
                    >
                      {msg.role === "assistant" ? (
                        <div className="prose prose-sm max-w-none">
                          <MarkdownRenderer content={msg.content} />
                        </div>
                      ) : (
                        <p className="text-sm whitespace-pre-wrap">
                          {msg.content}
                        </p>
                      )}
                      <p
                        className={`text-[10px] mt-1.5 ${
                          msg.role === "user"
                            ? "text-brand-200"
                            : "text-gray-400"
                        }`}
                      >
                        {new Date(msg.created_at).toLocaleTimeString()}
                      </p>
                    </div>
                    {msg.role === "user" && (
                      <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-gray-200">
                        <User className="h-4 w-4 text-gray-600" />
                      </div>
                    )}
                  </div>
                ))}
              </div>
            )}

            {sendMessage.isPending && (
              <div className="flex gap-3">
                <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-brand-100">
                  <Bot className="h-4 w-4 text-brand-700" />
                </div>
                <div className="card flex items-center gap-2">
                  <Loader2 className="h-4 w-4 animate-spin text-brand-600" />
                  <p className="text-sm text-gray-500">Thinking...</p>
                </div>
              </div>
            )}

            {sendMessage.error && (
              <div className="rounded-lg border border-red-200 bg-red-50 p-3 text-sm text-red-700">
                Failed to send message: {String(sendMessage.error)}
              </div>
            )}

            <div ref={chatEndRef} />
          </div>
        </div>

        {/* Chat input */}
        {plan.status === "completed" && (
          <div className="border-t border-gray-200 bg-white px-6 py-4">
            <div className="mx-auto max-w-4xl">
              <div className="flex items-end gap-3">
                <textarea
                  value={input}
                  onChange={(e) => setInput(e.target.value)}
                  onKeyDown={handleKeyDown}
                  placeholder="Ask questions about the plans, compare options, discuss tradeoffs..."
                  className="input min-h-[44px] max-h-[120px] resize-none flex-1"
                  rows={1}
                  disabled={sendMessage.isPending}
                />
                <button
                  className="btn-primary shrink-0 h-[44px]"
                  onClick={handleSend}
                  disabled={!input.trim() || sendMessage.isPending}
                >
                  {sendMessage.isPending ? (
                    <Loader2 className="h-4 w-4 animate-spin" />
                  ) : (
                    <Send className="h-4 w-4" />
                  )}
                </button>
              </div>
              <p className="text-[10px] text-gray-400 mt-1.5">
                Press Enter to send. Use "Add Another Option" above to generate a new plan.
              </p>
            </div>
          </div>
        )}
      </div>
    </>
  );
}

function MarkdownRenderer({ content }: { content: string }) {
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
