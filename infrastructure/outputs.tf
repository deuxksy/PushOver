# Note: Worker URL will be available after wrangler deployment
# output "worker_url" {
#   description = "Worker URL"
#   value       = "https://${var.worker_name}.workers.dev"
# }

output "d1_database_id" {
  description = "D1 Database ID"
  value       = cloudflare_d1_database.pushover.id
}

output "kv_namespace_id" {
  description = "KV Namespace ID"
  value       = cloudflare_workers_kv_namespace.cache.id
}
