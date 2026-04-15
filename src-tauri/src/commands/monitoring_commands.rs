use crate::db::AppDb;
use crate::models::project::Project;
use crate::commands::deploy_commands::resolve_infra_dir;
use serde::Serialize;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Instant;
use tokio::process::Command;

// ── Types ──

#[derive(Debug, Clone, Serialize)]
pub struct InstanceStatus {
    pub state: String,
    pub instance_id: String,
    pub instance_type: String,
    pub public_ip: String,
    pub launch_time: String,
    pub availability_zone: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AppHealth {
    pub healthy: bool,
    pub status_code: u16,
    pub response_ms: u64,
    pub url: String,
    pub checked_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MetricDatapoint {
    pub timestamp: String,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CloudWatchMetrics {
    pub metric_name: String,
    pub datapoints: Vec<MetricDatapoint>,
    pub unit: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContainerInfo {
    pub name: String,
    pub image: String,
    pub status: String,
    pub ports: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServiceCost {
    pub service: String,
    pub amount: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CostSummary {
    pub total: String,
    pub currency: String,
    pub period_start: String,
    pub period_end: String,
    pub by_service: Vec<ServiceCost>,
}

// ── Helpers ──

fn build_aws_cmd(args: &[&str], region: &str) -> Command {
    let aws_path = crate::tools::aws::resolve_aws_path_pub();
    // Use the AWS executable directly (not via cmd /C which breaks on spaces in paths)
    let mut cmd = Command::new(&aws_path);
    for a in args { cmd.arg(a); }
    cmd.arg("--region").arg(region);
    cmd.arg("--output").arg("json");
    cmd.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
    cmd
}

async fn run_aws_cmd(args: &[&str], region: &str) -> Result<serde_json::Value, String> {
    let output = build_aws_cmd(args, region)
        .output()
        .await
        .map_err(|e| format!("Failed to run AWS CLI: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("AWS CLI error: {}", stderr.trim()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout)
        .map_err(|e| format!("Failed to parse AWS response: {}", e))
}

fn get_project_blocking(state: &tauri::State<'_, AppDb>, project_id: &str) -> Result<Project, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    Project::get_by_id(&conn, project_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Project not found".to_string())
}

async fn get_instance_id(project: &Project) -> Result<String, String> {
    let infra_dir = resolve_infra_dir(project);
    // Try tofu output first
    if let Ok(id) = crate::commands::deploy_commands::get_tofu_output_sensitive_pub(&infra_dir, "instance_id").await {
        if !id.is_empty() {
            return Ok(id);
        }
    }

    // Fallback: find by project tag
    let result = run_aws_cmd(
        &["ec2", "describe-instances",
          "--filters", &format!("Name=tag:Project,Values={}", project.name),
          "Name=instance-state-name,Values=running"],
        &project.aws_region,
    ).await?;

    result["Reservations"]
        .as_array()
        .and_then(|r| r.first())
        .and_then(|r| r["Instances"].as_array())
        .and_then(|i| i.first())
        .and_then(|i| i["InstanceId"].as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "No running instance found for this project".to_string())
}

async fn get_public_ip(project: &Project) -> Result<String, String> {
    let infra_dir = resolve_infra_dir(project);
    if let Ok(ip) = crate::commands::deploy_commands::get_tofu_output_sensitive_pub(&infra_dir, "static_ip").await {
        if !ip.is_empty() {
            return Ok(ip);
        }
    }
    Err("Could not determine public IP".to_string())
}

// ── Commands ──

#[tauri::command]
pub async fn get_instance_status(
    state: tauri::State<'_, AppDb>,
    project_id: String,
) -> Result<InstanceStatus, String> {
    let project = get_project_blocking(&state, &project_id)?;
    let instance_id = get_instance_id(&project).await?;

    let result = run_aws_cmd(
        &["ec2", "describe-instances", "--instance-ids", &instance_id],
        &project.aws_region,
    ).await?;

    let instance = result["Reservations"]
        .as_array()
        .and_then(|r| r.first())
        .and_then(|r| r["Instances"].as_array())
        .and_then(|i| i.first())
        .ok_or("Instance not found in AWS response")?;

    Ok(InstanceStatus {
        state: instance["State"]["Name"].as_str().unwrap_or("unknown").to_string(),
        instance_id: instance["InstanceId"].as_str().unwrap_or("").to_string(),
        instance_type: instance["InstanceType"].as_str().unwrap_or("").to_string(),
        public_ip: instance["PublicIpAddress"].as_str().unwrap_or("").to_string(),
        launch_time: instance["LaunchTime"].as_str().unwrap_or("").to_string(),
        availability_zone: instance["Placement"]["AvailabilityZone"].as_str().unwrap_or("").to_string(),
    })
}

#[tauri::command]
pub async fn get_app_health(
    state: tauri::State<'_, AppDb>,
    project_id: String,
) -> Result<AppHealth, String> {
    let project = get_project_blocking(&state, &project_id)?;
    let ip = get_public_ip(&project).await?;
    let url = format!("http://{}", ip);
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    let start = Instant::now();

    // Use curl for the health check (works on all platforms)
    let output = if cfg!(windows) {
        Command::new("cmd")
            .args(["/C", "curl", "-s", "-o", "NUL", "-w", "%{http_code}", &url, "--connect-timeout", "5", "--max-time", "10"])
            .stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped())
            .output().await
    } else {
        Command::new("curl")
            .args(["-s", "-o", "/dev/null", "-w", "%{http_code}", &url, "--connect-timeout", "5", "--max-time", "10"])
            .stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped())
            .output().await
    };

    let elapsed = start.elapsed().as_millis() as u64;

    match output {
        Ok(out) => {
            let code_str = String::from_utf8_lossy(&out.stdout).trim().to_string();
            let code: u16 = code_str.parse().unwrap_or(0);
            Ok(AppHealth {
                healthy: code >= 200 && code < 400,
                status_code: code,
                response_ms: elapsed,
                url,
                checked_at: now,
            })
        }
        Err(_) => Ok(AppHealth {
            healthy: false,
            status_code: 0,
            response_ms: elapsed,
            url,
            checked_at: now,
        }),
    }
}

#[tauri::command]
pub async fn get_cloudwatch_metrics(
    state: tauri::State<'_, AppDb>,
    project_id: String,
    metric_name: String,
    hours: u32,
) -> Result<CloudWatchMetrics, String> {
    let project = get_project_blocking(&state, &project_id)?;
    let instance_id = get_instance_id(&project).await?;

    let end = chrono::Utc::now();
    let start = end - chrono::Duration::hours(hours as i64);
    let start_str = start.format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let end_str = end.format("%Y-%m-%dT%H:%M:%SZ").to_string();

    let dimension = format!("Name=InstanceId,Value={}", instance_id);
    let stat = if metric_name == "CPUUtilization" { "Average" } else { "Sum" };

    let result = run_aws_cmd(
        &["cloudwatch", "get-metric-statistics",
          "--namespace", "AWS/EC2",
          "--metric-name", &metric_name,
          "--dimensions", &dimension,
          "--start-time", &start_str,
          "--end-time", &end_str,
          "--period", "300",
          "--statistics", stat],
        &project.aws_region,
    ).await?;

    let datapoints = result["Datapoints"]
        .as_array()
        .map(|arr| {
            let mut points: Vec<MetricDatapoint> = arr.iter().filter_map(|dp| {
                let ts = dp["Timestamp"].as_str()?.to_string();
                let val = dp[stat].as_f64()?;
                Some(MetricDatapoint { timestamp: ts, value: val })
            }).collect();
            points.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
            points
        })
        .unwrap_or_default();

    let unit = result["Datapoints"]
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|dp| dp["Unit"].as_str())
        .unwrap_or("None")
        .to_string();

    Ok(CloudWatchMetrics {
        metric_name,
        datapoints,
        unit,
    })
}

#[tauri::command]
pub async fn get_container_status(
    state: tauri::State<'_, AppDb>,
    project_id: String,
) -> Result<Vec<ContainerInfo>, String> {
    let project = get_project_blocking(&state, &project_id)?;
    let ip = get_public_ip(&project).await?;
    let infra_dir = resolve_infra_dir(&project);

    // Get SSH key
    let ssh_key = crate::commands::deploy_commands::get_tofu_output_sensitive_pub(&infra_dir, "ssh_private_key").await
        .map_err(|_| "Could not get SSH key".to_string())?;

    // Write temp key file
    let key_path = std::env::temp_dir().join(format!("operra-mon-{}", project.name));
    std::fs::write(&key_path, &ssh_key).map_err(|e| format!("Failed to write key: {}", e))?;
    #[cfg(unix)]
    { std::fs::set_permissions(&key_path, std::os::unix::fs::PermissionsExt::from_mode(0o600)).ok(); }

    let ssh_user = "ubuntu";
    let ssh_cmd = format!("docker ps --format '{{{{.Names}}}}|{{{{.Image}}}}|{{{{.Status}}}}|{{{{.Ports}}}}'");

    let output = if cfg!(windows) {
        Command::new("cmd")
            .args(["/C", "ssh",
                   "-i", &key_path.to_string_lossy(),
                   "-o", "StrictHostKeyChecking=no",
                   "-o", "ConnectTimeout=5",
                   &format!("{}@{}", ssh_user, ip),
                   &ssh_cmd])
            .stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped())
            .output().await
    } else {
        Command::new("ssh")
            .args(["-i", &key_path.to_string_lossy(),
                   "-o", "StrictHostKeyChecking=no",
                   "-o", "ConnectTimeout=5",
                   &format!("{}@{}", ssh_user, ip),
                   &ssh_cmd])
            .stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped())
            .output().await
    };

    let _ = std::fs::remove_file(&key_path);

    let out = output.map_err(|e| format!("SSH failed: {}", e))?;
    let stdout = String::from_utf8_lossy(&out.stdout);

    let containers: Vec<ContainerInfo> = stdout.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|line| {
            let parts: Vec<&str> = line.split('|').collect();
            ContainerInfo {
                name: parts.first().unwrap_or(&"").to_string(),
                image: parts.get(1).unwrap_or(&"").to_string(),
                status: parts.get(2).unwrap_or(&"").to_string(),
                ports: parts.get(3).unwrap_or(&"").to_string(),
            }
        })
        .collect();

    Ok(containers)
}

#[tauri::command]
pub async fn get_cost_summary(
    state: tauri::State<'_, AppDb>,
    project_id: String,
) -> Result<CostSummary, String> {
    let project = get_project_blocking(&state, &project_id)?;

    let now = chrono::Utc::now();
    let start = now.format("%Y-%m-01").to_string();
    let end = (now + chrono::Duration::days(1)).format("%Y-%m-%d").to_string();
    let period = format!("Start={},End={}", start, end);

    // Filter by Project tag to only show this project's costs
    let tag_filter = format!(
        r#"{{"Tags":{{"Key":"Project","Values":["{}"],"MatchOptions":["EQUALS"]}}}}"#,
        project.name
    );

    // Cost Explorer API is always in us-east-1
    let result = run_aws_cmd(
        &["ce", "get-cost-and-usage",
          "--time-period", &period,
          "--granularity", "MONTHLY",
          "--metrics", "UnblendedCost",
          "--group-by", "Type=DIMENSION,Key=SERVICE",
          "--filter", &tag_filter],
        "us-east-1",
    ).await?;

    let mut by_service = Vec::new();
    let mut total: f64 = 0.0;

    if let Some(results) = result["ResultsByTime"].as_array() {
        for period_result in results {
            if let Some(groups) = period_result["Groups"].as_array() {
                for group in groups {
                    let service = group["Keys"]
                        .as_array()
                        .and_then(|k| k.first())
                        .and_then(|k| k.as_str())
                        .unwrap_or("Unknown")
                        .to_string();
                    let amount = group["Metrics"]["UnblendedCost"]["Amount"]
                        .as_str()
                        .unwrap_or("0")
                        .parse::<f64>()
                        .unwrap_or(0.0);

                    if amount > 0.001 {
                        total += amount;
                        by_service.push(ServiceCost {
                            service,
                            amount: format!("{:.2}", amount),
                        });
                    }
                }
            }
        }
    }

    by_service.sort_by(|a, b| {
        b.amount.parse::<f64>().unwrap_or(0.0)
            .partial_cmp(&a.amount.parse::<f64>().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(CostSummary {
        total: format!("{:.2}", total),
        currency: "USD".to_string(),
        period_start: start,
        period_end: end,
        by_service,
    })
}
