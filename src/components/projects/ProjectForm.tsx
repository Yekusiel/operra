import { useState } from "react";
import { FolderOpen, Github, Monitor } from "lucide-react";
import { open } from "@tauri-apps/plugin-dialog";
import type { CreateProjectInput } from "../../lib/types";
import { AWS_REGIONS } from "../../lib/types";

interface ProjectFormProps {
  onSubmit: (input: CreateProjectInput) => void;
  isSubmitting: boolean;
  error?: string | null;
}

export function ProjectForm({ onSubmit, isSubmitting, error }: ProjectFormProps) {
  const [sourceType, setSourceType] = useState<"local" | "github">("local");
  const [name, setName] = useState("");
  const [repoPath, setRepoPath] = useState("");
  const [githubRepo, setGithubRepo] = useState("");
  const [githubBranch, setGithubBranch] = useState("main");
  const [awsMethod, setAwsMethod] = useState<"profile" | "keys">("profile");
  const [awsProfile, setAwsProfile] = useState("");
  const [awsAccessKeyId, setAwsAccessKeyId] = useState("");
  const [awsSecretAccessKey, setAwsSecretAccessKey] = useState("");
  const [awsRegion, setAwsRegion] = useState("us-east-1");
  const [domain, setDomain] = useState("");
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
    const input: CreateProjectInput = {
      name: name.trim(),
      source_type: sourceType,
      aws_region: awsRegion,
      domain: domain.trim() || undefined,
      description: description.trim() || undefined,
    };

    if (sourceType === "local") {
      input.repo_path = repoPath.trim();
    } else {
      input.github_repo = githubRepo.trim();
      input.github_branch = githubBranch.trim() || "main";
    }

    if (awsMethod === "profile") {
      input.aws_profile = awsProfile.trim() || undefined;
    } else {
      input.aws_access_key_id = awsAccessKeyId.trim() || undefined;
      input.aws_secret_access_key = awsSecretAccessKey.trim() || undefined;
    }

    onSubmit(input);
  };

  const isValid =
    name.trim() &&
    (sourceType === "local" ? repoPath.trim() : githubRepo.trim()) &&
    (awsMethod === "profile" || (awsAccessKeyId.trim() && awsSecretAccessKey.trim()));

  return (
    <form onSubmit={handleSubmit} className="space-y-6">
      {error && (
        <div className="rounded-lg border border-red-200 bg-red-50 p-4 text-sm text-red-700">
          {error}
        </div>
      )}

      {/* Source Type Selector */}
      <div>
        <label className="label">Project Source</label>
        <div className="grid grid-cols-2 gap-3">
          <button
            type="button"
            onClick={() => setSourceType("local")}
            className={`flex items-center gap-3 rounded-lg border px-4 py-3 text-left text-sm transition-colors ${
              sourceType === "local"
                ? "border-brand-500 bg-brand-50 text-brand-900"
                : "border-gray-200 bg-white text-gray-700 hover:border-gray-300"
            }`}
          >
            <Monitor className="h-5 w-5" />
            <div>
              <p className="font-medium">Local Directory</p>
              <p className="text-xs text-gray-500">A folder on this computer</p>
            </div>
          </button>
          <button
            type="button"
            onClick={() => setSourceType("github")}
            className={`flex items-center gap-3 rounded-lg border px-4 py-3 text-left text-sm transition-colors ${
              sourceType === "github"
                ? "border-brand-500 bg-brand-50 text-brand-900"
                : "border-gray-200 bg-white text-gray-700 hover:border-gray-300"
            }`}
          >
            <Github className="h-5 w-5" />
            <div>
              <p className="font-medium">GitHub Repository</p>
              <p className="text-xs text-gray-500">Clone from GitHub</p>
            </div>
          </button>
        </div>
      </div>

      {/* Source-specific fields */}
      {sourceType === "local" ? (
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
            <button type="button" className="btn-secondary shrink-0" onClick={handleBrowse}>
              <FolderOpen className="h-4 w-4" />
              Browse
            </button>
          </div>
        </div>
      ) : (
        <div className="space-y-4">
          <div>
            <label htmlFor="github_repo" className="label">
              GitHub Repository <span className="text-red-500">*</span>
            </label>
            <input
              id="github_repo"
              type="text"
              className="input"
              placeholder="owner/repo"
              value={githubRepo}
              onChange={(e) => {
                setGithubRepo(e.target.value);
                if (!name && e.target.value.includes("/")) {
                  setName(e.target.value.split("/").pop() || "");
                }
              }}
              required
            />
            <p className="mt-1 text-xs text-gray-500">
              e.g., Yekusiel/givinity
            </p>
          </div>
          <div>
            <label htmlFor="github_branch" className="label">Branch</label>
            <input
              id="github_branch"
              type="text"
              className="input"
              placeholder="main"
              value={githubBranch}
              onChange={(e) => setGithubBranch(e.target.value)}
            />
          </div>
        </div>
      )}

      {/* Project Name */}
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

      {/* AWS Credentials */}
      <div>
        <label className="label">AWS Credentials</label>
        <div className="grid grid-cols-2 gap-3 mb-3">
          <button
            type="button"
            onClick={() => setAwsMethod("profile")}
            className={`rounded-lg border px-3 py-2 text-xs text-left transition-colors ${
              awsMethod === "profile"
                ? "border-brand-500 bg-brand-50 text-brand-900"
                : "border-gray-200 bg-white text-gray-600 hover:border-gray-300"
            }`}
          >
            <p className="font-medium">Use AWS CLI Profile</p>
            <p className="text-gray-500">From ~/.aws/credentials</p>
          </button>
          <button
            type="button"
            onClick={() => setAwsMethod("keys")}
            className={`rounded-lg border px-3 py-2 text-xs text-left transition-colors ${
              awsMethod === "keys"
                ? "border-brand-500 bg-brand-50 text-brand-900"
                : "border-gray-200 bg-white text-gray-600 hover:border-gray-300"
            }`}
          >
            <p className="font-medium">Enter Access Keys</p>
            <p className="text-gray-500">Paste Key ID + Secret</p>
          </button>
        </div>

        {awsMethod === "profile" ? (
          <input
            type="text"
            className="input"
            placeholder="default"
            value={awsProfile}
            onChange={(e) => setAwsProfile(e.target.value)}
          />
        ) : (
          <div className="space-y-3">
            <div>
              <label htmlFor="aws_key" className="text-xs text-gray-600">
                Access Key ID <span className="text-red-500">*</span>
              </label>
              <input
                id="aws_key"
                type="text"
                className="input font-mono"
                placeholder="AKIA..."
                value={awsAccessKeyId}
                onChange={(e) => setAwsAccessKeyId(e.target.value)}
              />
            </div>
            <div>
              <label htmlFor="aws_secret" className="text-xs text-gray-600">
                Secret Access Key <span className="text-red-500">*</span>
              </label>
              <input
                id="aws_secret"
                type="password"
                className="input font-mono"
                placeholder="wJal..."
                value={awsSecretAccessKey}
                onChange={(e) => setAwsSecretAccessKey(e.target.value)}
              />
            </div>
          </div>
        )}
      </div>

      {/* Region */}
      <div>
        <label htmlFor="aws_region" className="label">AWS Region</label>
        <select
          id="aws_region"
          className="input"
          value={awsRegion}
          onChange={(e) => setAwsRegion(e.target.value)}
        >
          {AWS_REGIONS.map((r) => (
            <option key={r.value} value={r.value}>{r.label}</option>
          ))}
        </select>
      </div>

      {/* Domain */}
      <div>
        <label htmlFor="domain" className="label">Custom Domain (optional)</label>
        <input
          id="domain"
          type="text"
          className="input"
          placeholder="myapp.example.com"
          value={domain}
          onChange={(e) => setDomain(e.target.value)}
        />
        <p className="mt-1 text-xs text-gray-500">
          If you have a domain, enter it here. DNS instructions will be shown after deployment.
        </p>
      </div>

      {/* Description */}
      <div>
        <label htmlFor="description" className="label">Description</label>
        <textarea
          id="description"
          className="input min-h-[60px] resize-y"
          placeholder="Brief description of this project..."
          value={description}
          onChange={(e) => setDescription(e.target.value)}
          rows={2}
        />
      </div>

      {/* Submit */}
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
