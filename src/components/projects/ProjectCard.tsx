import { useNavigate } from "react-router-dom";
import { FolderGit2, Github, Cloud, Clock, ChevronRight } from "lucide-react";
import type { Project } from "../../lib/types";

interface ProjectCardProps {
  project: Project;
}

export function ProjectCard({ project }: ProjectCardProps) {
  const navigate = useNavigate();

  return (
    <button
      onClick={() => navigate(`/projects/${project.id}`)}
      className="card group flex w-full cursor-pointer items-start gap-4 text-left transition-shadow hover:shadow-md"
    >
      <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-brand-50">
        <FolderGit2 className="h-5 w-5 text-brand-600" />
      </div>

      <div className="min-w-0 flex-1">
        <div className="flex items-center justify-between">
          <h3 className="text-sm font-semibold text-gray-900 truncate">
            {project.name}
          </h3>
          <ChevronRight className="h-4 w-4 text-gray-400 opacity-0 transition-opacity group-hover:opacity-100" />
        </div>

        {project.description && (
          <p className="mt-0.5 text-sm text-gray-500 truncate">
            {project.description}
          </p>
        )}

        <div className="mt-3 flex flex-wrap items-center gap-3 text-xs text-gray-500">
          <span className="flex items-center gap-1 truncate" title={project.source_type === "github" ? project.github_repo || "" : project.repo_path}>
            {project.source_type === "github" ? (
              <><Github className="h-3.5 w-3.5" />{project.github_repo}</>
            ) : (
              <><FolderGit2 className="h-3.5 w-3.5" />{truncatePath(project.repo_path)}</>
            )}
          </span>

          {project.aws_profile && (
            <span className="flex items-center gap-1">
              <Cloud className="h-3.5 w-3.5" />
              {project.aws_profile} / {project.aws_region}
            </span>
          )}

          <span className="flex items-center gap-1 ml-auto">
            <Clock className="h-3.5 w-3.5" />
            {formatDate(project.created_at)}
          </span>
        </div>
      </div>
    </button>
  );
}

function truncatePath(path: string, maxLen = 40): string {
  if (path.length <= maxLen) return path;
  const parts = path.replace(/\\/g, "/").split("/");
  if (parts.length <= 3) return path;
  return `.../${parts.slice(-3).join("/")}`;
}

function formatDate(iso: string): string {
  try {
    return new Date(iso).toLocaleDateString(undefined, {
      month: "short",
      day: "numeric",
      year: "numeric",
    });
  } catch {
    return iso;
  }
}
