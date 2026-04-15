import { useState, useEffect } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { ArrowLeft, Save, Loader2, FolderOpen, Github, Monitor } from "lucide-react";
import { open } from "@tauri-apps/plugin-dialog";
import { TopBar } from "../components/layout/TopBar";
import { useProject, useUpdateProject } from "../hooks/useProjects";
import type { CreateProjectInput } from "../lib/types";
import { AWS_REGIONS } from "../lib/types";

export function ProjectSettingsPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { data: project, isLoading } = useProject(id!);
  const updateProject = useUpdateProject();

  const [name, setName] = useState("");
  const [sourceType, setSourceType] = useState<"local" | "github">("local");
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

  useEffect(() => {
    if (!project) return;
    setName(project.name);
    setSourceType(project.source_type as "local" | "github");
    setRepoPath(project.repo_path);
    setGithubRepo(project.github_repo || "");
    setGithubBranch(project.github_branch || "main");
    setAwsRegion(project.aws_region);
    setDomain(project.domain || "");
    setDescription(project.description || "");
    if (project.aws_access_key_id) {
      setAwsMethod("keys");
      setAwsAccessKeyId(project.aws_access_key_id);
      setAwsSecretAccessKey(project.aws_secret_access_key || "");
    } else {
      setAwsMethod("profile");
      setAwsProfile(project.aws_profile || "");
    }
  }, [project]);

  const handleBrowse = async () => {
    const selected = await open({ directory: true, multiple: false });
    if (selected) setRepoPath(selected as string);
  };

  const handleSave = () => {
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

    updateProject.mutate(
      { id: id!, input },
      { onSuccess: () => navigate(`/projects/${id}`) }
    );
  };

  if (isLoading || !project) {
    return (
      <div className="flex flex-1 items-center justify-center">
        <div className="h-8 w-8 animate-spin rounded-full border-2 border-brand-600 border-t-transparent" />
      </div>
    );
  }

  return (
    <>
      <TopBar
        title="Project Settings"
        subtitle={project.name}
        actions={
          <button className="btn-secondary" onClick={() => navigate(`/projects/${id}`)}>
            <ArrowLeft className="h-4 w-4" />
            Back
          </button>
        }
      />

      <div className="flex-1 p-6">
        <div className="mx-auto max-w-2xl">
          <div className="card space-y-6">
            {updateProject.error && (
              <div className="rounded-lg border border-red-200 bg-red-50 p-4 text-sm text-red-700">
                {String(updateProject.error)}
              </div>
            )}

            {/* Project Name */}
            <div>
              <label className="label">Project Name</label>
              <input
                type="text"
                className="input"
                value={name}
                onChange={(e) => setName(e.target.value)}
              />
            </div>

            {/* Source Type */}
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
                  </div>
                </button>
              </div>
            </div>

            {/* Source-specific fields */}
            {sourceType === "local" ? (
              <div>
                <label className="label">Repository Path</label>
                <div className="flex gap-2">
                  <input
                    type="text"
                    className="input flex-1"
                    value={repoPath}
                    onChange={(e) => setRepoPath(e.target.value)}
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
                  <label className="label">GitHub Repository</label>
                  <input
                    type="text"
                    className="input"
                    placeholder="owner/repo"
                    value={githubRepo}
                    onChange={(e) => setGithubRepo(e.target.value)}
                  />
                </div>
                <div>
                  <label className="label">Branch</label>
                  <input
                    type="text"
                    className="input"
                    value={githubBranch}
                    onChange={(e) => setGithubBranch(e.target.value)}
                  />
                </div>
              </div>
            )}

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
                  <p className="font-medium">AWS CLI Profile</p>
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
                  <p className="font-medium">Access Keys</p>
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
                    <label className="text-xs text-gray-600">Access Key ID</label>
                    <input
                      type="text"
                      className="input font-mono"
                      placeholder="AKIA..."
                      value={awsAccessKeyId}
                      onChange={(e) => setAwsAccessKeyId(e.target.value)}
                    />
                  </div>
                  <div>
                    <label className="text-xs text-gray-600">Secret Access Key</label>
                    <input
                      type="password"
                      className="input font-mono"
                      value={awsSecretAccessKey}
                      onChange={(e) => setAwsSecretAccessKey(e.target.value)}
                    />
                  </div>
                </div>
              )}
            </div>

            {/* Region */}
            <div>
              <label className="label">AWS Region</label>
              <select
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
              <label className="label">Custom Domain</label>
              <input
                type="text"
                className="input"
                placeholder="myapp.example.com"
                value={domain}
                onChange={(e) => setDomain(e.target.value)}
              />
              <p className="mt-1 text-xs text-gray-500">
                DNS instructions shown after deployment.
              </p>
            </div>

            {/* Description */}
            <div>
              <label className="label">Description</label>
              <textarea
                className="input min-h-[60px] resize-y"
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                rows={2}
              />
            </div>

            {/* Save */}
            <div className="flex justify-end gap-3 pt-2 border-t border-gray-200">
              <button className="btn-secondary" onClick={() => navigate(`/projects/${id}`)}>
                Cancel
              </button>
              <button
                className="btn-primary"
                onClick={handleSave}
                disabled={!name.trim() || updateProject.isPending}
              >
                {updateProject.isPending ? (
                  <><Loader2 className="h-4 w-4 animate-spin" /> Saving...</>
                ) : (
                  <><Save className="h-4 w-4" /> Save Changes</>
                )}
              </button>
            </div>
          </div>
        </div>
      </div>
    </>
  );
}
