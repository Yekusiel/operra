import { useState, useEffect } from "react";
import { useParams, useNavigate } from "react-router-dom";
import {
  ArrowLeft,
  ArrowRight,
  CheckCircle2,
  ClipboardList,
} from "lucide-react";
import { TopBar } from "../components/layout/TopBar";
import { useProject } from "../hooks/useProjects";
import {
  useQuestionnaire,
  useGetOrCreateQuestionnaire,
  useSaveQuestionnaire,
} from "../hooks/useQuestionnaire";
import type { ArchitectureAnswers } from "../lib/types";
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

  // Initialize questionnaire
  useEffect(() => {
    if (existing) {
      setQuestionnaireId(existing.id);
      try {
        const parsed = JSON.parse(existing.answers_json);
        setAnswers(parsed);
      } catch {
        // ignore parse errors
      }
    } else if (!getOrCreate.isPending && !getOrCreate.data) {
      getOrCreate.mutate(undefined, {
        onSuccess: (q) => {
          setQuestionnaireId(q.id);
          try {
            const parsed = JSON.parse(q.answers_json);
            setAnswers(parsed);
          } catch {
            // ignore
          }
        },
      });
    }
  }, [existing]);

  const question = ARCHITECTURE_QUESTIONS[currentStep];
  const totalSteps = ARCHITECTURE_QUESTIONS.length;
  const isLast = currentStep === totalSteps - 1;
  const isFirst = currentStep === 0;

  const currentValue = question ? answers[question.key] || "" : "";

  const handleSelect = (value: string) => {
    const updated = { ...answers, [question.key]: value };
    setAnswers(updated);

    // Auto-save draft
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
      // Complete the questionnaire
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
                {question.options.map((option) => (
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
                    {option.label}
                  </button>
                ))}
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
                    <span className="truncate">
                      {q.label}: {option?.label || val || "—"}
                    </span>
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
