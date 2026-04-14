output "app_url" {
  description = "Application URL"
  value       = "https://${var.domain_name}"
}

output "app_url_ip" {
  description = "Application URL via IP (HTTP only, for testing before DNS)"
  value       = "http://${aws_eip.app.public_ip}"
}

output "static_ip" {
  description = "Elastic IP address — point your DNS A record here"
  value       = aws_eip.app.public_ip
}

output "ssh_command" {
  description = "SSH command to connect to the instance"
  value       = "ssh -i ${var.project_name}-key.pem ec2-user@${aws_eip.app.public_ip}"
}

output "ssh_private_key" {
  description = "SSH private key (save to a .pem file)"
  value       = tls_private_key.ssh.private_key_openssh
  sensitive   = true
}

output "db_password" {
  description = "PostgreSQL database password"
  value       = random_password.db_password.result
  sensitive   = true
}

output "nextauth_secret" {
  description = "NextAuth.js secret"
  value       = random_password.nextauth_secret.result
  sensitive   = true
}

output "s3_backup_bucket" {
  description = "S3 bucket for database backups"
  value       = aws_s3_bucket.backups.id
}

output "deploy_command" {
  description = "Command to redeploy the application"
  value       = "ssh -i ${var.project_name}-key.pem ec2-user@${aws_eip.app.public_ip} 'sudo /opt/${var.project_name}/deploy.sh'"
}

output "dns_instructions" {
  description = "DNS configuration instructions"
  value       = <<-EOT
    To complete setup, create the following DNS record:

      Type: A
      Name: ${var.domain_name}
      Value: ${aws_eip.app.public_ip}
      TTL: 300

    Once DNS propagates, Caddy will automatically provision an HTTPS certificate via Let's Encrypt.
    You can test immediately via HTTP at: http://${aws_eip.app.public_ip}
  EOT
}