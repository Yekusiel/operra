import { Loader2 } from "lucide-react";
import type { ScanProgress as ScanProgressType } from "../../lib/types";

interface ScanProgressProps {
  progress: ScanProgressType | null;
}

export function ScanProgressIndicator({ progress }: ScanProgressProps) {
  if (!progress) return null;

  const phaseLabels: Record<string, string> = {
    walking: "Scanning files...",
    detecting: "Analyzing project structure...",
    complete: "Scan complete",
  };

  const label = phaseLabels[progress.phase] || progress.phase;
  const isComplete = progress.phase === "complete";

  return (
    <div className="card border-brand-200 bg-brand-50">
      <div className="flex items-center gap-3">
        {!isComplete && (
          <Loader2 className="h-5 w-5 animate-spin text-brand-600" />
        )}
        <div>
          <p className="text-sm font-medium text-brand-900">{label}</p>
          <p className="text-xs text-brand-700">
            {progress.files_checked.toLocaleString()} files checked
            {progress.detections_so_far > 0 &&
              ` · ${progress.detections_so_far} detections`}
          </p>
        </div>
      </div>
    </div>
  );
}
