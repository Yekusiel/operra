import { useState, useEffect } from "react";
import { useParams, useNavigate } from "react-router-dom";
import {
  ArrowLeft,
  ArrowRight,
  CheckCircle2,
  ClipboardList,
  Cpu,
} from "lucide-react";
import { TopBar } from "../components/layout/TopBar";
import { useProject } from "../hooks/useProjects";
import {
  useQuestionnaire,
  useGetOrCreateQuestionnaire,
  useSaveQuestionnaire,
} from "../hooks/useQuestionnaire";
import * as api from "../lib/tauri";
import type { ArchitectureAnswers, AutoFilledAnswers, AutoFillEntry } from "../lib/types";
import { ARCHITECTURE_QUESTIONS } from "../lib/types";

export function QuestionnairePage() {
  const { id: projectId } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { data: project } = useProject(projectId!);
  const { data: existing } = useQuestionnaire(projectId!);
  const getOrCreate = useGetOrCreateQuestionnaire(projectId!);
  const save = useSaveQuestionnaire(projectId!);

  const [questionnaireId, setQuestionnaireId] = useState<string | null>(null);
  const [currentStep, setCurrentStep] = useState(0);
  const [answers, setAnswers] = useState<ArchitectureAnswers>({});
  const [autofill, setAutofill] = useState<AutoFilledAnswers | null>(null);

  // Fetch autofill suggestions from scan findings
  useEffect(() => {
    api.getAutofillSuggestions(projectId!).then(setAutofill).catch(() => {});
  }, [projectId]);

  // Initialize questionnaire and apply autofill to empty fields
  useEffect(() => {
    if (existing) {
      setQuestionnaireId(existing.id);
      try {
        const parsed = JSON.parse(existing.answers_json) as ArchitectureAnswers;
        // Apply autofill only to fields the user hasn't already answered
        if (autofill) {
          const merged = { ...parsed };
          if (!merged.database_needs && autofill.database_needs) {
            merged.database_needs = autofill.database_needs.value;
          }
          if (!merged.background_jobs && autofill.background_jobs) {
            merged.background_jobs = autofill.background_jobs.value;
          }
          if (!merged.networking && autofill.networking) {
            merged.networking = autofill.networking.value;
          }
          if (!merged.storage_needs && autofill.storage_needs) {
            merged.storage_needs = autofill.storage_needs.value;
          }
          setAnswers(merged);
        } else {
          setAnswers(parsed);
        }
      } catch {
        // ignore
      }
    } else if (!getOrCreate.isPending && !getOrCreate.data) {
      getOrCreate.mutate(undefined, {
        onSuccess: (q) => {
          setQuestionnaireId(q.id);
          // Pre-fill from autofill for brand new questionnaire
          const initial: ArchitectureAnswers = {};
          if (autofill?.database_needs) initial.database_needs = autofill.database_needs.value;
          if (autofill?.background_jobs) initial.background_jobs = autofill.background_jobs.value;
          if (autofill?.networking) initial.networking = autofill.networking.value;
          if (autofill?.storage_needs) initial.storage_needs = autofill.storage_needs.value;
          setAnswers(initial);
        },
      });
    }
  }, [existing, autofill]);

  const question = ARCHITECTURE_QUESTIONS[currentStep];
  const totalSteps = ARCHITECTURE_QUESTIONS.length;
  const isLast = currentStep === totalSteps - 1;
  const isFirst = currentStep === 0;

  const currentValue = question ? answers[question.key] || "" : "";

  // Get autofill entry for current question if available
  const currentAutofill: AutoFillEntry | null =
    question && autofill
      ? (autofill as unknown as Record<string, AutoFillEntry | null>)[question.key] ?? null
      : null;

  const handleSelect = (value: string) => {
    const updated = { ...answers, [question.key]: value };
    setAnswers(updated);

    if (questionnaireId) {
      save.mutate({
        id: questionnaireId,
        answersJson: JSON.stringify(updated),
        completed: false,
      });
    }
  };

  const handleNext = () => {
    if (isLast) {
      if (questionnaireId) {
        save.mutate(
          {
            id: questionnaireId,
            answersJson: JSON.stringify(answers),
            completed: true,
          },
          {
            onSuccess: () => navigate(`/projects/${projectId}`),
          }
        );
      }
    } else {
      setCurrentStep((s) => s + 1);
    }
  };

  const handleBack = () => {
    if (isFirst) {
      navigate(`/projects/${projectId}`);
    } else {
      setCurrentStep((s) => s - 1);
    }
  };

  if (!question) return null;

  return (
    <>
      <TopBar
        title="Architecture Questionnaire"
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
        <div className="mx-auto max-w-2xl">
          {/* Progress bar */}
          <div className="mb-8">
            <div className="flex items-center justify-between text-sm text-gray-500 mb-2">
              <span>
                Question {currentStep + 1} of {totalSteps}
              </span>
              <span>
                {Object.values(answers).filter(Boolean).length} answered
              </span>
            </div>
            <div className="h-2 rounded-full bg-gray-200">
              <div
                className="h-2 rounded-full bg-brand-600 transition-all duration-300"
                style={{
                  width: `${((currentStep + 1) / totalSteps) * 100}%`,
                }}
              />
            </div>
          </div>

          {/* Auto-detected banner */}
          {currentAutofill && (
            <div className="mb-4 flex items-start gap-2 rounded-lg border border-brand-200 bg-brand-50 px-4 py-3">
              <Cpu className="h-4 w-4 text-brand-600 mt-0.5 shrink-0" />
              <div>
                <p className="text-xs font-medium text-brand-800">
                  Auto-detected from your codebase
                </p>
                <p className="text-xs text-brand-700 mt-0.5">
                  {currentAutofill.reason}
                </p>
                <p className="text-[10px] text-brand-600 mt-0.5">
                  Evidence: {currentAutofill.evidence.join(", ")}
                </p>
              </div>
            </div>
          )}

          {/* Question card */}
          <div className="card">
            <div className="flex items-start gap-3 mb-6">
              <ClipboardList className="h-5 w-5 text-brand-600 mt-0.5" />
              <div>
                <h2 className="text-base font-semibold text-gray-900">
                  {question.label}
                </h2>
                <p className="text-sm text-gray-500 mt-0.5">
                  {question.description}
                </p>
              </div>
            </div>

            {question.type === "select" && question.options && (
              <div className="space-y-2">
                {question.options.map((option) => {
                  const isAutoFilled =
                    currentAutofill?.value === option.value &&
                    currentValue === option.value;

                  return (
                    <button
                      key={option.value}
                      onClick={() => handleSelect(option.value)}
                      className={`flex w-full items-center gap-3 rounded-lg border px-4 py-3 text-left text-sm transition-colors ${
                        currentValue === option.value
                          ? "border-brand-500 bg-brand-50 text-brand-900"
                          : "border-gray-200 bg-white text-gray-700 hover:border-gray-300 hover:bg-gray-50"
                      }`}
                    >
                      <div
                        className={`flex h-5 w-5 shrink-0 items-center justify-center rounded-full border-2 ${
                          currentValue === option.value
                            ? "border-brand-600 bg-brand-600"
                            : "border-gray-300"
                        }`}
                      >
                        {currentValue === option.value && (
                          <CheckCircle2 className="h-3.5 w-3.5 text-white" />
                        )}
                      </div>
                      <span className="flex-1">{option.label}</span>
                      {isAutoFilled && (
                        <span className="badge-blue text-[10px]">
                          Auto-detected
                        </span>
                      )}
                    </button>
                  );
                })}
              </div>
            )}

            {question.type === "text" && (
              <textarea
                className="input min-h-[120px] resize-y"
                placeholder="Type your answer here..."
                value={currentValue}
                onChange={(e) => handleSelect(e.target.value)}
                rows={5}
              />
            )}
          </div>

          {/* Navigation */}
          <div className="mt-6 flex items-center justify-between">
            <button className="btn-secondary" onClick={handleBack}>
              <ArrowLeft className="h-4 w-4" />
              {isFirst ? "Cancel" : "Previous"}
            </button>

            <button className="btn-primary" onClick={handleNext}>
              {isLast ? (
                <>
                  <CheckCircle2 className="h-4 w-4" />
                  Complete
                </>
              ) : (
                <>
                  Next
                  <ArrowRight className="h-4 w-4" />
                </>
              )}
            </button>
          </div>

          {/* Quick answer overview */}
          <div className="mt-8 card">
            <h3 className="text-xs font-semibold uppercase tracking-wider text-gray-500 mb-3">
              Answers so far
            </h3>
            <div className="grid grid-cols-2 gap-2">
              {ARCHITECTURE_QUESTIONS.map((q, i) => {
                const val = answers[q.key];
                const option = q.options?.find((o) => o.value === val);
                const af = autofill
                  ? (autofill as unknown as Record<string, AutoFillEntry | null>)[q.key]
                  : null;
                const isAutoFilled = af?.value === val && !!val;

                return (
                  <button
                    key={q.key}
                    onClick={() => setCurrentStep(i)}
                    className={`flex items-center gap-2 rounded-lg px-3 py-2 text-left text-xs transition-colors ${
                      i === currentStep
                        ? "bg-brand-50 text-brand-700"
                        : "hover:bg-gray-50 text-gray-600"
                    }`}
                  >
                    <span
                      className={`h-1.5 w-1.5 shrink-0 rounded-full ${
                        val ? "bg-green-500" : "bg-gray-300"
                      }`}
                    />
                    <span className="truncate flex-1">
                      {q.label}: {option?.label || val || "\u2014"}
                    </span>
                    {isAutoFilled && (
                      <Cpu className="h-3 w-3 text-brand-500 shrink-0" />
                    )}
                  </button>
                );
              })}
            </div>
          </div>
        </div>
      </div>
    </>
  );
}
