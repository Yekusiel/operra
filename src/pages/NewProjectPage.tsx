import { useNavigate } from "react-router-dom";
import { ArrowLeft } from "lucide-react";
import { TopBar } from "../components/layout/TopBar";
import { ProjectForm } from "../components/projects/ProjectForm";
import { useCreateProject } from "../hooks/useProjects";

export function NewProjectPage() {
  const navigate = useNavigate();
  const createProject = useCreateProject();

  return (
    <>
      <TopBar
        title="New Project"
        subtitle="Connect a local repository to Operra"
        actions={
          <button className="btn-secondary" onClick={() => navigate("/")}>
            <ArrowLeft className="h-4 w-4" />
            Back
          </button>
        }
      />

      <div className="flex-1 p-6">
        <div className="mx-auto max-w-2xl">
          <div className="card">
            <ProjectForm
              onSubmit={(input) => {
                createProject.mutate(input, {
                  onSuccess: (project) => {
                    navigate(`/projects/${project.id}`);
                  },
                });
              }}
              isSubmitting={createProject.isPending}
              error={
                createProject.error ? String(createProject.error) : null
              }
            />
          </div>
        </div>
      </div>
    </>
  );
}
