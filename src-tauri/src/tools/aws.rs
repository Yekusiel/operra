use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tokio::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsIdentity {
    #[serde(rename = "Account")]
    pub account: String,
    #[serde(rename = "Arn")]
    pub arn: String,
    #[serde(rename = "UserId")]
    pub user_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AwsConnectionResult {
    pub connected: bool,
    pub identity: Option<AwsIdentity>,
    pub error: Option<String>,
}

fn resolve_aws_path() -> String {
    if cfg!(windows) {
        // AWS CLI v2 installs to Program Files but often isn't in PATH
        let candidates = [
            "aws",
            "C:\\Program Files\\Amazon\\AWSCLIV2\\aws.exe",
            "C:\\Program Files (x86)\\Amazon\\AWSCLIV2\\aws.exe",
        ];
        for candidate in &candidates {
            let path = std::path::Path::new(candidate);
            if candidate.contains('\\') && path.exists() {
                return candidate.to_string();
            }
        }
        // Also check LOCALAPPDATA
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            let p = std::path::PathBuf::from(&local).join("Programs\\Amazon\\AWSCLIV2\\aws.exe");
            if p.exists() {
                return p.to_string_lossy().to_string();
            }
        }
        "aws".to_string()
    } else {
        "aws".to_string()
    }
}

fn build_aws_command(args: &[&str], profile: Option<&str>, region: Option<&str>) -> Command {
    let aws_path = resolve_aws_path();

    if cfg!(windows) {
        let mut cmd = Command::new("cmd");
        cmd.arg("/C").arg(&aws_path);
        for arg in args {
            cmd.arg(arg);
        }
        if let Some(p) = profile {
            cmd.arg("--profile").arg(p);
        }
        if let Some(r) = region {
            cmd.arg("--region").arg(r);
        }
        cmd
    } else {
        let mut cmd = Command::new(&aws_path);
        for arg in args {
            cmd.arg(arg);
        }
        if let Some(p) = profile {
            cmd.arg("--profile").arg(p);
        }
        if let Some(r) = region {
            cmd.arg("--region").arg(r);
        }
        cmd
    }
}

pub async fn test_connection(
    profile: Option<&str>,
    region: Option<&str>,
) -> AwsConnectionResult {
    let mut cmd = build_aws_command(
        &["sts", "get-caller-identity", "--output", "json"],
        profile,
        region,
    );

    let result = cmd
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await;

    match result {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                match serde_json::from_str::<AwsIdentity>(&stdout) {
                    Ok(identity) => AwsConnectionResult {
                        connected: true,
                        identity: Some(identity),
                        error: None,
                    },
                    Err(e) => AwsConnectionResult {
                        connected: false,
                        identity: None,
                        error: Some(format!("Failed to parse AWS identity: {}", e)),
                    },
                }
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                let error = if stderr.contains("could not be found") || stderr.contains("not found") {
                    "AWS CLI is not installed. Install it first.".to_string()
                } else if stderr.contains("Unable to locate credentials") || stderr.contains("NoCredentialProviders") {
                    format!(
                        "No AWS credentials found{}. Run 'aws configure{}' to set up your credentials.",
                        profile.map(|p| format!(" for profile '{}'", p)).unwrap_or_default(),
                        profile.map(|p| format!(" --profile {}", p)).unwrap_or_default(),
                    )
                } else if stderr.contains("InvalidClientTokenId") || stderr.contains("SignatureDoesNotMatch") {
                    "AWS credentials are invalid or expired. Check your access key and secret.".to_string()
                } else if stderr.contains("ExpiredToken") {
                    "AWS session token has expired. Refresh your credentials.".to_string()
                } else {
                    stderr
                };

                AwsConnectionResult {
                    connected: false,
                    identity: None,
                    error: Some(error),
                }
            }
        }
        Err(e) => {
            let error = if e.kind() == std::io::ErrorKind::NotFound {
                "AWS CLI is not installed. Install it to connect your AWS account.".to_string()
            } else {
                format!("Failed to run AWS CLI: {}", e)
            };
            AwsConnectionResult {
                connected: false,
                identity: None,
                error: Some(error),
            }
        }
    }
}

/// List available AWS CLI profiles from ~/.aws/credentials and ~/.aws/config
pub async fn list_profiles() -> Vec<String> {
    let mut cmd = build_aws_command(&["configure", "list-profiles"], None, None);

    let result = cmd
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await;

    match result {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect()
        }
        _ => vec![],
    }
}
