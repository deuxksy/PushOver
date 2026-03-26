terraform {
  required_version = ">= 1.0"
  required_providers {
    cloudflare = {
      source  = "cloudflare/cloudflare"
      version = "~> 4.50"
    }
  }
}

provider "cloudflare" {
  api_token = var.cloudflare_api_token
  # Note: email is not required when using api_token
}

# Note: Worker deployment is handled via wrangler
# Terraform manages D1, KV, and routing infrastructure

# D1 Database
resource "cloudflare_d1_database" "pushover" {
  account_id = var.account_id
  name       = "pushover-db"
}

# KV Namespace (optional - for caching)
resource "cloudflare_workers_kv_namespace" "cache" {
  account_id = var.account_id
  title       = "pushover-cache"
}

# Worker Route (requires manual script creation via wrangler first)
# Uncomment after deploying worker with wrangler
# resource "cloudflare_worker_route" "pushover" {
#   zone_id     = var.zone_id
#   pattern     = "api.pushover.example.com/*"
#   script_name = var.worker_name
# }
