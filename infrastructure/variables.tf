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

# ============================================
# R2 Backend Credentials
# ============================================
# Used for Terraform state storage in Cloudflare R2
# Generate via: Cloudflare Dashboard → R2 → Manage R2 API Tokens

variable "r2_access_key_id" {
  type        = string
  description = "R2 Access Key ID for Terraform State Backend"
  sensitive   = true
}

variable "r2_secret_access_key" {
  type        = string
  description = "R2 Secret Access Key for Terraform State Backend"
  sensitive   = true
}
