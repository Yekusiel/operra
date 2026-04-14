import { useState } from "react";
import { FolderOpen } from "lucide-react";
import { open } from "@tauri-apps/plugin-dialog";
import type { CreateProjectInput } from "../../lib/types";
import { AWS_REGIONS } from "../../lib/types";

interface ProjectFormProps {
  onSubmit: (input: CreateProjectInput) => void;
  isSubmitting: boolean;
  error?: string | null;
}

export function ProjectForm({ onSubmit, isSubmitting, error }: ProjectFormProps) {
  const [name, setName] = useState("");
  const [repoPath, setRepoPath] = useState("");
  const [awsProfile, setAwsProfile] = useState("");
  const [awsRegion, setAwsRegion] = useState("us-east-1");
  const [description, setDescription] = useState("");

  const handleBrowse = async () => {
    const selected = await open({ directory: true, multiple: false });
    if (selected) {
      setRepoPath(selected as string);
      if (!name) {
        const parts = (selected as string).replace(/\\/g, "/").split("/");
        setName(parts[parts.length - 1] || "");
      }
    }
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onSubmit({
      name: name.trim(),
      repo_path: repoPath.trim(),
      aws_profile: awsProfile.trim() || undefined,
      aws_region: awsRegion || undefined,
      description: description.trim() || undefined,
    });
  };

  const isValid = name.trim() && repoPath.trim();

  return (
    <form onSubmit={handleSubmit} className="space-y-6">
      {error && (
        <div className="rounded-lg border border-red-200 bg-red-50 p-4 text-sm text-red-700">
          {error}
        </div>
      )}

      <div>
        <label htmlFor="repo_path" className="label">
          Repository Path <span className="text-red-500">*</span>
        </label>
        <div className="flex gap-2">
          <input
            id="repo_path"
            type="text"
            className="input flex-1"
            placeholder="/path/to/your/project"
            value={repoPath}
            onChange={(e) => setRepoPath(e.target.value)}
            required
          />
          <button
            type="button"
            className="btn-secondary shrink-0"
            onClick={handleBrowse}
          >
            <FolderOpen className="h-4 w-4" />
            Browse
          </button>
        </div>
        <p className="mt-1 text-xs text-gray-500">
          Select the root directory of your project repository.
        </p>
      </div>

      <div>
        <label htmlFor="name" className="label">
          Project Name <span className="text-red-500">*</span>
        </label>
        <input
          id="name"
          type="text"
          className="input"
          placeholder="my-awesome-app"
          value={name}
          onChange={(e) => setName(e.target.value)}
          required
        />
      </div>

      <div className="grid grid-cols-2 gap-4">
        <div>
          <label htmlFor="aws_profile" className="label">
            AWS Profile
          </label>
          <input
            id="aws_profile"
            type="text"
            className="input"
            placeholder="default"
            value={awsProfile}
            onChange={(e) => setAwsProfile(e.target.value)}
          />
          <p className="mt-1 text-xs text-gray-500">
            AWS CLI profile name from ~/.aws/credentials
          </p>
        </div>

        <div>
          <label htmlFor="aws_region" className="label">
            AWS Region
          </label>
          <select
            id="aws_region"
            className="input"
            value={awsRegion}
            onChange={(e) => setAwsRegion(e.target.value)}
          >
            {AWS_REGIONS.map((r) => (
              <option key={r.value} value={r.value}>
                {r.label}
              </option>
            ))}
          </select>
        </div>
      </div>

      <div>
        <label htmlFor="description" className="label">
          Description
        </label>
        <textarea
          id="description"
          className="input min-h-[80px] resize-y"
          placeholder="Brief description of this project..."
          value={description}
          onChange={(e) => setDescription(e.target.value)}
          rows={3}
        />
      </div>

      <div className="flex justify-end gap-3 pt-2">
        <button
          type="submit"
          className="btn-primary"
          disabled={!isValid || isSubmitting}
        >
          {isSubmitting ? (
            <>
              <div className="h-4 w-4 animate-spin rounded-full border-2 border-white border-t-transparent" />
              Creating...
            </>
          ) : (
            "Create Project"
          )}
        </button>
      </div>
    </form>
  );
}
