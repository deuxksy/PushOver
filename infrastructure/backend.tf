# ============================================
# Terraform Backend (Cloudflare R2)
# ============================================
# S3-compatible backend using Cloudflare R2
# State file is stored in R2 bucket managed by this Terraform project
#
# Initial setup procedure:
# 1. mv backend.tf backend.tf.bak
# 2. tofu init -backend=false
# 3. tofu apply -target=cloudflare_r2_bucket.terraform_state
# 4. mv backend.tf.bak backend.tf
# 5. tofu init -reconfigure (answer 'yes' to copy state)
# ============================================

terraform {
  backend "s3" {
    bucket = "pushover-terraform-state"
    key    = "terraform.tfstate"
    region = "auto"

    endpoints = {
      s3 = "https://e0924c382d21ac0f10aee606b82687ce.r2.cloudflarestorage.com"
    }

    skip_credentials_validation = true
    skip_metadata_api_check     = true
    skip_region_validation      = true
    skip_requesting_account_id  = true
  }
}
