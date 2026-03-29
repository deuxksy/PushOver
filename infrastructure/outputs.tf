# ============================================
# Outputs
# ============================================

output "d1_database_id" {
  description = "D1 Database ID"
  value       = cloudflare_d1_database.pushover.id
}

output "d1_database_uuid" {
  description = "D1 Database UUID"
  value       = cloudflare_d1_database.pushover.id
}

output "kv_namespace_id" {
  description = "KV Namespace ID"
  value       = cloudflare_workers_kv_namespace.cache.id
}

output "account_id" {
  description = "Cloudflare Account ID"
  value       = var.account_id
}

output "worker_script_name" {
  description = "Worker Script Name"
  value       = var.worker_name
}
