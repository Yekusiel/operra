variable "project_name" {
  description = "Project name used for resource naming and tagging"
  type        = string
  default     = "givinity"
}

variable "aws_region" {
  description = "AWS region to deploy into"
  type        = string
  default     = "us-east-2"
}

variable "aws_profile" {
  description = "AWS CLI profile to use"
  type        = string
  default     = "default"
}

variable "instance_type" {
  description = "EC2 instance type"
  type        = string
  default     = "t4g.micro"
}

variable "ebs_volume_size" {
  description = "Root EBS volume size in GB"
  type        = number
  default     = 20
}

variable "github_repo_url" {
  description = "GitHub repository URL to clone"
  type        = string
  default     = "https://github.com/Yekusiel/givinity.git"
}

variable "github_branch" {
  description = "Git branch to deploy"
  type        = string
  default     = "main"
}

variable "domain_name" {
  description = "Custom domain name for the application"
  type        = string
  default     = "givinity.app"
}

variable "app_port" {
  description = "Port the Next.js application listens on"
  type        = number
  default     = 3000
}

variable "db_name" {
  description = "PostgreSQL database name"
  type        = string
  default     = "givinity"
}

variable "db_user" {
  description = "PostgreSQL database user"
  type        = string
  default     = "givinity"
}