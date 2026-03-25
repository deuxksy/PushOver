terraform {
  required_version = ">= 1.0"
  required_providers {
    cloudflare = {
      source  = "cloudflare/cloudflare"
      version = "~> 4.0"
    }
  }
}

provider "cloudflare" {
  api_token = var.cloudflare_api_token
}

variable "cloudflare_api_token" {
  type      = string
  sensitive = true
}

variable "account_id" {
  type = string
}

variable "worker_name" {
  type    = string
  default = "pushover-worker"
}

# Cloudflare Worker
resource "cloudflare_worker_script" "pushover" {
  account_id = var.account_id
  name       = var.worker_name
  content    = filebase64("${path.module}/../crates/worker/build/worker/shim.mjs")
}

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

# Worker Route
resource "cloudflare_worker_route" "pushover" {
  zone_id     = var.zone_id
  pattern     = "api.pushover.example.com/*"
  script_name = cloudflare_worker_script.pushover.name
}

variable "zone_id" {
  type = string
}

# D1 Database Binding (via worker script settings)
# Note: This is managed via wrangler.toml in development
# In production, use cloudflare_workers_secret for sensitive values
