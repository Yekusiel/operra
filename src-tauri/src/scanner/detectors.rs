use super::types::{Detection, FindingCategory};
use std::path::Path;

/// Check if a file at `repo_root/relative_path` exists.
fn file_exists(repo_root: &Path, relative: &str) -> bool {
    repo_root.join(relative).exists()
}

/// Read a file's contents, returning None if it doesn't exist or can't be read.
fn read_file(repo_root: &Path, relative: &str) -> Option<String> {
    std::fs::read_to_string(repo_root.join(relative)).ok()
}

/// Check if package.json contains a dependency (in dependencies or devDependencies).
fn package_json_has_dep(pkg_content: &str, dep: &str) -> bool {
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(pkg_content) {
        for section in ["dependencies", "devDependencies"] {
            if let Some(deps) = parsed.get(section).and_then(|d| d.as_object()) {
                if deps.contains_key(dep) {
                    return true;
                }
            }
        }
    }
    false
}

/// Run all detectors against a repo root and return findings.
pub fn detect_all(repo_root: &Path) -> Vec<Detection> {
    let mut detections = Vec::new();

    let pkg_json = read_file(repo_root, "package.json");

    // === Languages ===

    if pkg_json.is_some() {
        detections.push(Detection {
            category: FindingCategory::Language,
            name: "Node.js".to_string(),
            confidence: 0.9,
            evidence_path: "package.json".to_string(),
            metadata: None,
        });
    }

    if file_exists(repo_root, "tsconfig.json")
        || pkg_json
            .as_ref()
            .map_or(false, |p| package_json_has_dep(p, "typescript"))
    {
        detections.push(Detection {
            category: FindingCategory::Language,
            name: "TypeScript".to_string(),
            confidence: 0.95,
            evidence_path: if file_exists(repo_root, "tsconfig.json") {
                "tsconfig.json"
            } else {
                "package.json"
            }
            .to_string(),
            metadata: None,
        });
    }

    if file_exists(repo_root, "requirements.txt")
        || file_exists(repo_root, "pyproject.toml")
        || file_exists(repo_root, "Pipfile")
        || file_exists(repo_root, "setup.py")
    {
        let evidence = ["requirements.txt", "pyproject.toml", "Pipfile", "setup.py"]
            .iter()
            .find(|f| file_exists(repo_root, f))
            .unwrap_or(&"requirements.txt");

        detections.push(Detection {
            category: FindingCategory::Language,
            name: "Python".to_string(),
            confidence: 0.9,
            evidence_path: evidence.to_string(),
            metadata: None,
        });
    }

    if file_exists(repo_root, "go.mod") {
        detections.push(Detection {
            category: FindingCategory::Language,
            name: "Go".to_string(),
            confidence: 0.95,
            evidence_path: "go.mod".to_string(),
            metadata: None,
        });
    }

    if file_exists(repo_root, "Cargo.toml") {
        detections.push(Detection {
            category: FindingCategory::Language,
            name: "Rust".to_string(),
            confidence: 0.95,
            evidence_path: "Cargo.toml".to_string(),
            metadata: None,
        });
    }

    if file_exists(repo_root, "pom.xml") || file_exists(repo_root, "build.gradle") {
        let evidence = if file_exists(repo_root, "pom.xml") {
            "pom.xml"
        } else {
            "build.gradle"
        };
        detections.push(Detection {
            category: FindingCategory::Language,
            name: "Java".to_string(),
            confidence: 0.9,
            evidence_path: evidence.to_string(),
            metadata: None,
        });
    }

    if file_exists(repo_root, "composer.json") {
        detections.push(Detection {
            category: FindingCategory::Language,
            name: "PHP".to_string(),
            confidence: 0.9,
            evidence_path: "composer.json".to_string(),
            metadata: None,
        });
    }

    // === Frameworks ===

    if let Some(ref pkg) = pkg_json {
        if package_json_has_dep(pkg, "next") || file_exists(repo_root, "next.config.js") || file_exists(repo_root, "next.config.mjs") || file_exists(repo_root, "next.config.ts") {
            detections.push(Detection {
                category: FindingCategory::Framework,
                name: "Next.js".to_string(),
                confidence: 0.95,
                evidence_path: "package.json".to_string(),
                metadata: None,
            });
        } else if package_json_has_dep(pkg, "react") {
            detections.push(Detection {
                category: FindingCategory::Framework,
                name: "React".to_string(),
                confidence: 0.9,
                evidence_path: "package.json".to_string(),
                metadata: None,
            });
        }

        if package_json_has_dep(pkg, "express") {
            detections.push(Detection {
                category: FindingCategory::Framework,
                name: "Express".to_string(),
                confidence: 0.9,
                evidence_path: "package.json".to_string(),
                metadata: None,
            });
        }

        if package_json_has_dep(pkg, "fastify") {
            detections.push(Detection {
                category: FindingCategory::Framework,
                name: "Fastify".to_string(),
                confidence: 0.9,
                evidence_path: "package.json".to_string(),
                metadata: None,
            });
        }
    }

    // Django detection
    if file_exists(repo_root, "manage.py") {
        if let Some(content) = read_file(repo_root, "manage.py") {
            if content.contains("django") {
                detections.push(Detection {
                    category: FindingCategory::Framework,
                    name: "Django".to_string(),
                    confidence: 0.95,
                    evidence_path: "manage.py".to_string(),
                    metadata: None,
                });
            }
        }
    }

    // Flask detection
    if let Some(ref req) = read_file(repo_root, "requirements.txt") {
        if req.to_lowercase().contains("flask") {
            detections.push(Detection {
                category: FindingCategory::Framework,
                name: "Flask".to_string(),
                confidence: 0.85,
                evidence_path: "requirements.txt".to_string(),
                metadata: None,
            });
        }
    }

    // WordPress detection
    if file_exists(repo_root, "wp-config.php") || file_exists(repo_root, "wp-config-sample.php") {
        detections.push(Detection {
            category: FindingCategory::Framework,
            name: "WordPress".to_string(),
            confidence: 0.95,
            evidence_path: if file_exists(repo_root, "wp-config.php") {
                "wp-config.php"
            } else {
                "wp-config-sample.php"
            }
            .to_string(),
            metadata: None,
        });
    }

    // === Infrastructure ===

    // Terraform / OpenTofu
    if has_tf_files(repo_root) {
        detections.push(Detection {
            category: FindingCategory::Infrastructure,
            name: "Terraform/OpenTofu".to_string(),
            confidence: 0.95,
            evidence_path: "*.tf".to_string(),
            metadata: None,
        });
    }

    // CDK
    if file_exists(repo_root, "cdk.json") {
        detections.push(Detection {
            category: FindingCategory::Infrastructure,
            name: "AWS CDK".to_string(),
            confidence: 0.95,
            evidence_path: "cdk.json".to_string(),
            metadata: None,
        });
    }

    // CloudFormation
    for template_file in ["template.yaml", "template.yml", "template.json"] {
        if let Some(content) = read_file(repo_root, template_file) {
            if content.contains("AWSTemplateFormatVersion") {
                detections.push(Detection {
                    category: FindingCategory::Infrastructure,
                    name: "CloudFormation".to_string(),
                    confidence: 0.95,
                    evidence_path: template_file.to_string(),
                    metadata: None,
                });
                break;
            }
        }
    }

    // SAM
    if file_exists(repo_root, "samconfig.toml") || file_exists(repo_root, "samconfig.yaml") {
        let evidence = if file_exists(repo_root, "samconfig.toml") {
            "samconfig.toml"
        } else {
            "samconfig.yaml"
        };
        detections.push(Detection {
            category: FindingCategory::Infrastructure,
            name: "AWS SAM".to_string(),
            confidence: 0.9,
            evidence_path: evidence.to_string(),
            metadata: None,
        });
    }

    // === Config ===

    if file_exists(repo_root, "Dockerfile") {
        detections.push(Detection {
            category: FindingCategory::Config,
            name: "Docker".to_string(),
            confidence: 0.95,
            evidence_path: "Dockerfile".to_string(),
            metadata: None,
        });
    }

    if file_exists(repo_root, "docker-compose.yml")
        || file_exists(repo_root, "docker-compose.yaml")
        || file_exists(repo_root, "compose.yml")
        || file_exists(repo_root, "compose.yaml")
    {
        let evidence = ["docker-compose.yml", "docker-compose.yaml", "compose.yml", "compose.yaml"]
            .iter()
            .find(|f| file_exists(repo_root, f))
            .unwrap_or(&"docker-compose.yml");

        detections.push(Detection {
            category: FindingCategory::Config,
            name: "Docker Compose".to_string(),
            confidence: 0.95,
            evidence_path: evidence.to_string(),
            metadata: None,
        });
    }

    if file_exists(repo_root, ".env.example") || file_exists(repo_root, ".env.sample") {
        detections.push(Detection {
            category: FindingCategory::Config,
            name: "Environment Config".to_string(),
            confidence: 0.7,
            evidence_path: ".env.example".to_string(),
            metadata: None,
        });
    }

    // === CI/CD ===

    if repo_root.join(".github/workflows").is_dir() {
        detections.push(Detection {
            category: FindingCategory::CiCd,
            name: "GitHub Actions".to_string(),
            confidence: 0.95,
            evidence_path: ".github/workflows/".to_string(),
            metadata: None,
        });
    }

    if file_exists(repo_root, ".gitlab-ci.yml") {
        detections.push(Detection {
            category: FindingCategory::CiCd,
            name: "GitLab CI".to_string(),
            confidence: 0.95,
            evidence_path: ".gitlab-ci.yml".to_string(),
            metadata: None,
        });
    }

    if file_exists(repo_root, "Jenkinsfile") {
        detections.push(Detection {
            category: FindingCategory::CiCd,
            name: "Jenkins".to_string(),
            confidence: 0.9,
            evidence_path: "Jenkinsfile".to_string(),
            metadata: None,
        });
    }

    if file_exists(repo_root, "buildspec.yml") || file_exists(repo_root, "buildspec.yaml") {
        detections.push(Detection {
            category: FindingCategory::CiCd,
            name: "AWS CodeBuild".to_string(),
            confidence: 0.9,
            evidence_path: "buildspec.yml".to_string(),
            metadata: None,
        });
    }

    detections
}

/// Check for .tf files in root or common subdirectories.
fn has_tf_files(repo_root: &Path) -> bool {
    let dirs_to_check = [
        "",
        "infra",
        "infrastructure",
        "terraform",
        "tofu",
        "iac",
    ];

    for dir in &dirs_to_check {
        let check_dir = if dir.is_empty() {
            repo_root.to_path_buf()
        } else {
            repo_root.join(dir)
        };

        if check_dir.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&check_dir) {
                for entry in entries.flatten() {
                    if let Some(ext) = entry.path().extension() {
                        if ext == "tf" {
                            return true;
                        }
                    }
                }
            }
        }
    }

    false
}

/// Infer the overall application stack from detections.
pub fn infer_stack(detections: &[Detection]) -> Option<String> {
    let names: Vec<&str> = detections.iter().map(|d| d.name.as_str()).collect();

    if names.contains(&"WordPress") {
        return Some("WordPress/PHP Application".to_string());
    }
    if names.contains(&"Next.js") {
        if names.contains(&"Docker") {
            return Some("Containerized Next.js Application".to_string());
        }
        return Some("Next.js Application".to_string());
    }
    if names.contains(&"Django") || names.contains(&"Flask") {
        if names.contains(&"Docker") {
            return Some("Containerized Python API".to_string());
        }
        return Some("Python API".to_string());
    }
    if names.contains(&"Express") || names.contains(&"Fastify") {
        if names.contains(&"Docker") {
            return Some("Containerized Node.js Backend".to_string());
        }
        return Some("Node.js Backend".to_string());
    }
    if names.contains(&"React") && !names.contains(&"Express") && !names.contains(&"Fastify") {
        return Some("Static React Application".to_string());
    }
    if names.contains(&"Go") {
        if names.contains(&"Docker") {
            return Some("Containerized Go Application".to_string());
        }
        return Some("Go Application".to_string());
    }
    if names.contains(&"Docker") {
        return Some("Dockerized Application".to_string());
    }
    if names.contains(&"Node.js") {
        return Some("Node.js Application".to_string());
    }
    if names.contains(&"Python") {
        return Some("Python Application".to_string());
    }

    None
}
