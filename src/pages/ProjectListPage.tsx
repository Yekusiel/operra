import { useNavigate } from "react-router-dom";
import { Plus, FolderKanban } from "lucide-react";
import { TopBar } from "../components/layout/TopBar";
import { ProjectCard } from "../components/projects/ProjectCard";
import { useProjects } from "../hooks/useProjects";

export function ProjectListPage() {
  const navigate = useNavigate();
  const { data: projects, isLoading, error } = useProjects();

  return (
    <>
      <TopBar
        title="Projects"
        subtitle="Manage your infrastructure projects"
        actions={
          <button
            className="btn-primary"
            onClick={() => navigate("/projects/new")}
          >
            <Plus className="h-4 w-4" />
            New Project
          </button>
        }
      />

      <div className="flex-1 p-6">
        {isLoading && (
          <div className="flex items-center justify-center py-20">
            <div className="h-8 w-8 animate-spin rounded-full border-2 border-brand-600 border-t-transparent" />
          </div>
        )}

        {error && (
          <div className="rounded-lg border border-red-200 bg-red-50 p-4 text-sm text-red-700">
            Failed to load projects: {String(error)}
          </div>
        )}

        {projects && projects.length === 0 && (
          <div className="flex flex-col items-center justify-center py-20">
            <div className="flex h-16 w-16 items-center justify-center rounded-2xl bg-gray-100">
              <FolderKanban className="h-8 w-8 text-gray-400" />
            </div>
            <h3 className="mt-4 text-sm font-semibold text-gray-900">
              No projects yet
            </h3>
            <p className="mt-1 text-sm text-gray-500">
              Create your first project to start managing infrastructure.
            </p>
            <button
              className="btn-primary mt-6"
              onClick={() => navigate("/projects/new")}
            >
              <Plus className="h-4 w-4" />
              New Project
            </button>
          </div>
        )}

        {projects && projects.length > 0 && (
          <div className="grid gap-4 sm:grid-cols-1 lg:grid-cols-2 xl:grid-cols-3">
            {projects.map((project) => (
              <ProjectCard key={project.id} project={project} />
            ))}
          </div>
        )}
      </div>
    </>
  );
}
