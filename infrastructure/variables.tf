variable "cloudflare_api_token" {
  type        = string
  description = "Cloudflare API Token with Workers and D1 permissions"
  sensitive   = true
}

variable "account_id" {
  type        = string
  description = "Cloudflare Account ID"
}

variable "zone_id" {
  type        = string
  description = "Cloudflare Zone ID for custom domain"
  default     = null
}

variable "worker_name" {
  type        = string
  description = "Worker script name"
  default     = "pushover-worker"
}

variable "environment" {
  type        = string
  description = "Environment name (dev, staging, prod)"
  default     = "dev"
}
