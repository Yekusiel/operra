mod adapters;
mod commands;
mod db;
mod models;
mod provisioning;
mod scanner;
mod tools;

use db::AppDb;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");
            std::fs::create_dir_all(&app_data_dir)?;
            let db = AppDb::init(&app_data_dir)?;
            app.manage(db);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Projects
            commands::project_commands::create_project,
            commands::project_commands::list_projects,
            commands::project_commands::get_project,
            commands::project_commands::update_project,
            commands::project_commands::delete_project,
            // Scanner
            commands::scan_commands::start_scan,
            commands::scan_commands::get_scan_results,
            commands::scan_commands::list_scans_for_project,
            // Questionnaire
            commands::questionnaire_commands::get_or_create_questionnaire,
            commands::questionnaire_commands::save_questionnaire,
            commands::questionnaire_commands::get_questionnaire,
            commands::questionnaire_commands::reset_questionnaire,
            commands::questionnaire_commands::get_autofill_suggestions,
            // Plans
            commands::plan_commands::generate_plan,
            commands::plan_commands::get_plan,
            commands::plan_commands::get_latest_plan,
            commands::plan_commands::list_plans,
            commands::plan_commands::generate_additional_option,
            commands::plan_commands::approve_plan,
            commands::plan_commands::get_approved_plan,
            commands::plan_commands::list_plan_options,
            commands::plan_commands::approve_plan_option,
            commands::plan_commands::get_approved_option,
            commands::plan_commands::send_plan_message,
            commands::plan_commands::get_plan_messages,
            // Tools & AWS
            commands::tools_commands::check_dependencies,
            commands::tools_commands::test_aws_connection,
            commands::tools_commands::get_aws_connection,
            commands::tools_commands::list_aws_profiles,
            // Deployment
            commands::deploy_commands::generate_iac,
            commands::deploy_commands::run_tofu_plan,
            commands::deploy_commands::approve_deployment,
            commands::deploy_commands::run_tofu_apply,
            commands::deploy_commands::get_deployment,
            commands::deploy_commands::list_deployments,
            commands::deploy_commands::destroy_infrastructure,
            // Monitoring
            commands::monitoring_commands::get_instance_status,
            commands::monitoring_commands::get_app_health,
            commands::monitoring_commands::get_cloudwatch_metrics,
            commands::monitoring_commands::get_container_status,
            commands::monitoring_commands::get_cost_summary,
            commands::deploy_commands::get_dns_instructions,
            commands::deploy_commands::get_deploy_key_info,
            commands::deploy_commands::get_cicd_secrets,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
