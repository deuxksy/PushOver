# ============================================
# PushOver Serverless Infrastructure
# ============================================
# Cloudflare Terraform Provider v4.x (v5 has stability issues)
# Worker deployment is managed via wrangler
# Terraform manages: D1, KV, Queue, Cron Trigger
# ============================================

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
}

# ============================================
# D1 Database
# ============================================
resource "cloudflare_d1_database" "pushover" {
  account_id = var.account_id
  name       = "pushover-db"
}

# ============================================
# KV Namespace (for failed message backup)
# ============================================
resource "cloudflare_workers_kv_namespace" "cache" {
  account_id = var.account_id
  title      = "pushover-cache"
}

# ============================================
# R2 Bucket (Message Images)
# ============================================
resource "cloudflare_r2_bucket" "pushover_images" {
  account_id = var.account_id
  name       = "pushover-images"
  location   = "APAC"
}

# ============================================
# R2 Bucket (D1 Backup Snapshots)
# ============================================
resource "cloudflare_r2_bucket" "pushover_backups" {
  account_id = var.account_id
  name       = "pushover-backups"
  location   = "APAC"
}

# ============================================
# Queue (Message Processing)
# ============================================
resource "cloudflare_queue" "messages" {
  account_id = var.account_id
  name       = "pushover-messages"
}

# ============================================
# Worker Route (optional - for custom domain)
# ============================================
# Uncomment after deploying worker with wrangler
# resource "cloudflare_worker_route" "pushover" {
#   zone_id     = var.zone_id
#   pattern     = "api.pushover.example.com/*"
#   script_name = var.worker_name
# }

# ============================================
# Notes on Worker Deployment
# ============================================
# The following resources are managed via wrangler.toml:
# - Worker Script (Rust/WASM)
# - Queue (cloudflare_queues)
# - Queue Consumer
# - Cron Trigger (*/5 * * * *)
#
# To deploy:
#   cd crates/worker && wrangler deploy
#
# Terraform manages infrastructure, wrangler manages code deployment
# This hybrid approach avoids v5 provider stability issues

# ============================================
# R2 Bucket (Terraform State Storage)
# ============================================
# state 버킷은 Terraform이 자신의 state를 저장하는 곳
# 자기 자신을 관리하면 destroy 시 state 유실 → 수동 관리
# 생성: wrangler r2 bucket create pushover-terraform-state --location WNAM

# ============================================
# Cron Trigger (Recovery Worker)
# ============================================
# Cron Trigger는 wrangler.toml에서 관리
# Worker 배포 시 자동으로 설정됨 (triggers.crons)
