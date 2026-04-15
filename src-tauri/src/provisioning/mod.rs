pub mod templates;

use serde::{Deserialize, Serialize};

/// Configuration values that get injected into provisioning templates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisioningConfig {
    pub project_name: String,
    pub source_type: String,         // "local" or "github"
    pub github_repo: String,         // "owner/repo"
    pub github_branch: String,       // "main"
    pub domain: String,              // "myapp.com" or empty
    pub aws_region: String,          // "us-east-2"
    pub db_name: String,
    pub db_user: String,
    pub ssh_user: String,            // "ubuntu" for Ubuntu AMIs
    pub app_port: u16,               // 3000 for Next.js
    pub node_version: String,        // "20"
    pub stack_type: String,          // from template selection
    pub has_docker_compose: bool,
    pub has_prisma: bool,
    pub deploy_key_ssm_path: String, // "/project/deploy-key"
}

/// Detected stack from scan findings that determines which template to use.
#[derive(Debug, Clone, PartialEq)]
pub enum StackType {
    NextjsWithDockerCompose,  // Next.js app + docker-compose for DB
    NextjsStandalone,         // Next.js app, no Docker
    NodeApiDocker,            // Node.js API with Dockerfile
    NodeApiStandalone,        // Node.js API, no Docker
    StaticSite,               // React/Vue/static build
    DockerCompose,            // Full Docker Compose stack with app service
    Unknown,
}

impl StackType {
    pub fn as_str(&self) -> &'static str {
        match self {
            StackType::NextjsWithDockerCompose => "nextjs-docker-compose",
            StackType::NextjsStandalone => "nextjs-standalone",
            StackType::NodeApiDocker => "node-api-docker",
            StackType::NodeApiStandalone => "node-api-standalone",
            StackType::StaticSite => "static-site",
            StackType::DockerCompose => "docker-compose-full",
            StackType::Unknown => "unknown",
        }
    }
}

/// Detect stack type from scan findings.
pub fn detect_stack_type(findings: &[crate::models::scan::ScanFinding]) -> StackType {
    let names: Vec<&str> = findings.iter().map(|f| f.name.as_str()).collect();
    let categories: Vec<&str> = findings.iter().map(|f| f.category.as_str()).collect();

    let has_nextjs = names.contains(&"Next.js");
    let has_react = names.contains(&"React");
    let has_express = names.contains(&"Express") || names.contains(&"Fastify");
    let has_docker = names.contains(&"Docker");
    let has_docker_compose = names.contains(&"Docker Compose");
    let has_nodejs = names.contains(&"Node.js");

    if has_nextjs && has_docker_compose {
        StackType::NextjsWithDockerCompose
    } else if has_nextjs {
        StackType::NextjsStandalone
    } else if has_express && has_docker {
        StackType::NodeApiDocker
    } else if has_express || (has_nodejs && !has_react) {
        StackType::NodeApiStandalone
    } else if has_react && !has_express && !has_nextjs {
        StackType::StaticSite
    } else if has_docker_compose && has_docker {
        StackType::DockerCompose
    } else {
        StackType::Unknown
    }
}
