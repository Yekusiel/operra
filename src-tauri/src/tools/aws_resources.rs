/// Returns valid AWS Terraform/OpenTofu resource types for common services.
/// This prevents the AI from hallucinating resource type names.

pub fn get_valid_resources_for_services(plan_text: &str) -> Vec<(&'static str, Vec<&'static str>)> {
    let text = plan_text.to_lowercase();
    let mut services = Vec::new();

    if text.contains("lightsail") {
        services.push(("Lightsail", vec![
            "aws_lightsail_instance",
            "aws_lightsail_static_ip",
            "aws_lightsail_static_ip_attachment",
            "aws_lightsail_key_pair",
            "aws_lightsail_domain",
            "aws_lightsail_domain_entry",
            "aws_lightsail_database",
            "aws_lightsail_container_service",
            "aws_lightsail_container_service_deployment_version",
            "aws_lightsail_disk",
            "aws_lightsail_disk_attachment",
            "aws_lightsail_lb",
            "aws_lightsail_lb_attachment",
            "aws_lightsail_lb_certificate",
            "aws_lightsail_lb_stickiness_policy",
            "aws_lightsail_certificate",
            "aws_lightsail_bucket",
            "aws_lightsail_bucket_resource_access",
            "aws_lightsail_distribution",
        ]));
    }

    if text.contains("ec2") || text.contains("instance") || text.contains("compute") {
        services.push(("EC2", vec![
            "aws_instance",
            "aws_ami",
            "aws_ebs_volume",
            "aws_volume_attachment",
            "aws_key_pair",
            "aws_eip",
            "aws_eip_association",
            "aws_launch_template",
            "aws_placement_group",
        ]));
    }

    if text.contains("ecs") || text.contains("fargate") || text.contains("container") {
        services.push(("ECS", vec![
            "aws_ecs_cluster",
            "aws_ecs_service",
            "aws_ecs_task_definition",
            "aws_ecs_capacity_provider",
            "aws_ecs_cluster_capacity_providers",
        ]));
    }

    if text.contains("lambda") || text.contains("serverless") {
        services.push(("Lambda", vec![
            "aws_lambda_function",
            "aws_lambda_alias",
            "aws_lambda_event_source_mapping",
            "aws_lambda_function_url",
            "aws_lambda_invocation",
            "aws_lambda_layer_version",
            "aws_lambda_permission",
            "aws_lambda_provisioned_concurrency_config",
        ]));
    }

    if text.contains("api gateway") || text.contains("apigateway") || text.contains("api_gateway") {
        services.push(("API Gateway v2", vec![
            "aws_apigatewayv2_api",
            "aws_apigatewayv2_authorizer",
            "aws_apigatewayv2_deployment",
            "aws_apigatewayv2_domain_name",
            "aws_apigatewayv2_integration",
            "aws_apigatewayv2_route",
            "aws_apigatewayv2_stage",
            "aws_apigatewayv2_vpc_link",
        ]));
    }

    if text.contains("rds") || text.contains("postgres") || text.contains("mysql") || text.contains("database") || text.contains("aurora") {
        services.push(("RDS", vec![
            "aws_db_instance",
            "aws_db_subnet_group",
            "aws_db_parameter_group",
            "aws_db_option_group",
            "aws_rds_cluster",
            "aws_rds_cluster_instance",
            "aws_rds_cluster_parameter_group",
        ]));
    }

    if text.contains("dynamodb") {
        services.push(("DynamoDB", vec![
            "aws_dynamodb_table",
            "aws_dynamodb_table_item",
            "aws_dynamodb_global_table",
            "aws_dynamodb_contributor_insights",
            "aws_dynamodb_kinesis_streaming_destination",
        ]));
    }

    if text.contains("s3") || text.contains("bucket") || text.contains("storage") {
        services.push(("S3", vec![
            "aws_s3_bucket",
            "aws_s3_bucket_acl",
            "aws_s3_bucket_cors_configuration",
            "aws_s3_bucket_lifecycle_configuration",
            "aws_s3_bucket_logging",
            "aws_s3_bucket_notification",
            "aws_s3_bucket_object",
            "aws_s3_bucket_ownership_controls",
            "aws_s3_bucket_policy",
            "aws_s3_bucket_public_access_block",
            "aws_s3_bucket_server_side_encryption_configuration",
            "aws_s3_bucket_versioning",
            "aws_s3_bucket_website_configuration",
            "aws_s3_object",
        ]));
    }

    if text.contains("cloudfront") || text.contains("cdn") {
        services.push(("CloudFront", vec![
            "aws_cloudfront_distribution",
            "aws_cloudfront_origin_access_identity",
            "aws_cloudfront_origin_access_control",
            "aws_cloudfront_cache_policy",
            "aws_cloudfront_function",
            "aws_cloudfront_response_headers_policy",
        ]));
    }

    if text.contains("alb") || text.contains("load balancer") || text.contains("elb") || text.contains("target group") {
        services.push(("Load Balancing", vec![
            "aws_lb",
            "aws_lb_listener",
            "aws_lb_listener_rule",
            "aws_lb_target_group",
            "aws_lb_target_group_attachment",
        ]));
    }

    if text.contains("vpc") || text.contains("subnet") || text.contains("network") || text.contains("security group") {
        services.push(("VPC / Networking", vec![
            "aws_vpc",
            "aws_subnet",
            "aws_internet_gateway",
            "aws_nat_gateway",
            "aws_route_table",
            "aws_route_table_association",
            "aws_route",
            "aws_security_group",
            "aws_security_group_rule",
            "aws_vpc_security_group_ingress_rule",
            "aws_vpc_security_group_egress_rule",
            "aws_network_acl",
            "aws_eip",
        ]));
    }

    if text.contains("iam") || text.contains("role") || text.contains("policy") {
        services.push(("IAM", vec![
            "aws_iam_role",
            "aws_iam_role_policy",
            "aws_iam_role_policy_attachment",
            "aws_iam_policy",
            "aws_iam_instance_profile",
            "aws_iam_user",
            "aws_iam_user_policy",
            "aws_iam_user_policy_attachment",
            "aws_iam_group",
            "aws_iam_group_policy",
            "aws_iam_group_policy_attachment",
        ]));
    }

    if text.contains("acm") || text.contains("certificate") || text.contains("ssl") || text.contains("tls") {
        services.push(("ACM", vec![
            "aws_acm_certificate",
            "aws_acm_certificate_validation",
        ]));
    }

    if text.contains("route53") || text.contains("route 53") || text.contains("dns") {
        services.push(("Route 53", vec![
            "aws_route53_zone",
            "aws_route53_record",
            "aws_route53_health_check",
        ]));
    }

    if text.contains("cloudwatch") || text.contains("monitoring") || text.contains("alarm") || text.contains("logs") {
        services.push(("CloudWatch", vec![
            "aws_cloudwatch_log_group",
            "aws_cloudwatch_log_stream",
            "aws_cloudwatch_metric_alarm",
            "aws_cloudwatch_dashboard",
        ]));
    }

    if text.contains("sqs") || text.contains("queue") {
        services.push(("SQS", vec![
            "aws_sqs_queue",
            "aws_sqs_queue_policy",
            "aws_sqs_queue_redrive_policy",
            "aws_sqs_queue_redrive_allow_policy",
        ]));
    }

    if text.contains("sns") || text.contains("notification") {
        services.push(("SNS", vec![
            "aws_sns_topic",
            "aws_sns_topic_policy",
            "aws_sns_topic_subscription",
        ]));
    }

    if text.contains("ecr") || text.contains("container registry") {
        services.push(("ECR", vec![
            "aws_ecr_repository",
            "aws_ecr_lifecycle_policy",
            "aws_ecr_repository_policy",
        ]));
    }

    if text.contains("elasticache") || text.contains("redis") || text.contains("memcached") {
        services.push(("ElastiCache", vec![
            "aws_elasticache_cluster",
            "aws_elasticache_replication_group",
            "aws_elasticache_subnet_group",
            "aws_elasticache_parameter_group",
        ]));
    }

    if text.contains("auto scaling") || text.contains("autoscaling") || text.contains("asg") {
        services.push(("Auto Scaling", vec![
            "aws_autoscaling_group",
            "aws_autoscaling_policy",
            "aws_autoscaling_schedule",
            "aws_autoscaling_attachment",
        ]));
    }

    if text.contains("secrets manager") || text.contains("secret") {
        services.push(("Secrets Manager", vec![
            "aws_secretsmanager_secret",
            "aws_secretsmanager_secret_version",
        ]));
    }

    if text.contains("ssm") || text.contains("parameter store") {
        services.push(("SSM", vec![
            "aws_ssm_parameter",
        ]));
    }

    // Always include data sources that are commonly needed
    services.push(("Common Data Sources", vec![
        "data.aws_availability_zones",
        "data.aws_caller_identity",
        "data.aws_region",
        "data.aws_ami",
        "data.aws_iam_policy_document",
    ]));

    services
}

/// Format the valid resources into a string for the prompt.
pub fn format_valid_resources(plan_text: &str) -> String {
    let services = get_valid_resources_for_services(plan_text);
    if services.is_empty() {
        return String::new();
    }

    let mut output = String::from("## Valid AWS Resource Types\n\nYou MUST ONLY use resource types from this list. Do NOT use any resource type not listed here.\n\n");

    for (service_name, resources) in &services {
        output.push_str(&format!("### {}\n", service_name));
        for resource in resources {
            output.push_str(&format!("- `{}`\n", resource));
        }
        output.push('\n');
    }

    output
}
