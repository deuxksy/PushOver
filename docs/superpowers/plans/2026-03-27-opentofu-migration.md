# OpenTofu 마이그레이션 구현 계획

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Cloudflare Cron Trigger를 wrangler에서 Terraform으로 마이그레이션하여 인프라를 코드로 관리

**Architecture:** Terraform v4 Provider로 R2(State), D1, KV, Cron Trigger 관리. Worker 코드 배포는 wrangler 유지. 하이브리드 접근으로 v5 불안정성 회피.

**Tech Stack:** OpenTofu/Terraform, Cloudflare Provider v4.50, R2 Backend, wrangler

---

## File Structure

```
infrastructure/
├── backend.tf           # R2 backend 설정 (신규)
├── main.tf              # 메인 리소스 (수정: R2, Cron Trigger 추가)
├── variables.tf         # 입력 변수 (수정: R2 credentials 추가)
├── outputs.tf           # 출력값 (수정: queue_name 추가)
├── terraform.tfvars     # 변수 값 (수정: R2 credentials 추가)
└── .terraform.lock.hcl  # provider 버전 고정
```

**wrangler.toml 변경:**
- `[triggers]` 섹션 제거 (Terraform 관리)

---

## Task 1: R2 Backend 설정 파일 생성

**Files:**
- Create: `infrastructure/backend.tf`

- [ ] **Step 1: backend.tf 생성**

```hcl
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
```

- [ ] **Step 2: 파일 생성 확인**

Run: `cat infrastructure/backend.tf | head -5`
Expected: `# ============================================`

---

## Task 2: R2 Bucket 및 Cron Trigger 리소스 추가

**Files:**
- Modify: `infrastructure/main.tf`

- [ ] **Step 1: main.tf에 R2 Bucket 리소스 추가**

`cloudflare_workers_kv_namespace` 리소스 뒤에 다음 내용 추가:

```hcl
# ============================================
# R2 Bucket (Terraform State Storage)
# ============================================
resource "cloudflare_r2_bucket" "terraform_state" {
  account_id = var.account_id
  name       = "pushover-terraform-state"
  location   = "auto"
}

# ============================================
# Cron Trigger (Recovery Worker)
# ============================================
# Runs every 5 minutes to process failed messages
resource "cloudflare_workers_cron_trigger" "recovery" {
  account_id  = var.account_id
  script_name = var.worker_name

  schedules = [
    {
      cron = "*/5 * * * *"
    }
  ]
}
```

- [ ] **Step 2: main.tf 수정 확인**

Run: `grep -A 10 "cloudflare_r2_bucket" infrastructure/main.tf`
Expected: `resource "cloudflare_r2_bucket" "terraform_state"`

---

## Task 3: R2 Credentials 변수 추가

**Files:**
- Modify: `infrastructure/variables.tf`

- [ ] **Step 1: variables.tf에 R2 변수 추가**

파일 끝에 다음 내용 추가:

```hcl
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
```

- [ ] **Step 2: 변수 추가 확인**

Run: `grep "r2_access_key_id" infrastructure/variables.tf`
Expected: `variable "r2_access_key_id"`

---

## Task 4: Outputs 업데이트

**Files:**
- Modify: `infrastructure/outputs.tf`

- [ ] **Step 1: outputs.tf 전체 교체**

```hcl
# ============================================
# Outputs
# ============================================

output "d1_database_id" {
  description = "D1 Database ID"
  value       = cloudflare_d1_database.pushover.id
}

output "kv_namespace_id" {
  description = "KV Namespace ID"
  value       = cloudflare_workers_kv_namespace.cache.id
}

output "r2_bucket_name" {
  description = "R2 Bucket Name for Terraform State"
  value       = cloudflare_r2_bucket.terraform_state.name
}

output "cron_trigger_schedules" {
  description = "Cron Trigger Schedules"
  value       = cloudflare_workers_cron_trigger.recovery.schedules
}

output "worker_script_name" {
  description = "Worker Script Name"
  value       = var.worker_name
}
```

- [ ] **Step 2: outputs 수정 확인**

Run: `grep "cron_trigger_schedules" infrastructure/outputs.tf`
Expected: `output "cron_trigger_schedules"`

---

## Task 5: terraform.tfvars에 R2 credentials 추가

**Files:**
- Modify: `infrastructure/terraform.tfvars`

- [ ] **Step 1: terraform.tfvars에 R2 변수 값 추가**

파일 끝에 다음 내용 추가:

```hcl
# R2 Credentials for Terraform State Backend
# Generate via: Cloudflare Dashboard → R2 → Manage R2 API Tokens
r2_access_key_id     = "<R2_ACCESS_KEY_ID>"
r2_secret_access_key = "<R2_SECRET_ACCESS_KEY>"
```

- [ ] **Step 2: .gitignore에 terraform.tfvars 포함 확인**

Run: `grep "terraform.tfvars" .gitignore`
Expected: `terraform.tfvars` 또는 `*.tfvars`

---

## Task 6: Terraform 초기화 및 R2 Bucket 생성

**Files:**
- None (runtime)

- [ ] **Step 1: backend.tf 임시 비활성화**

```bash
cd infrastructure
mv backend.tf backend.tf.bak
```

Run: `ls infrastructure/backend.tf.bak`
Expected: `infrastructure/backend.tf.bak`

- [ ] **Step 2: 로컬 backend로 초기화**

Run: `cd infrastructure && tofu init -backend=false`
Expected: `OpenTofu has been successfully initialized!`

- [ ] **Step 3: 기존 리소스 import**

Run: `cd infrastructure && tofu import cloudflare_d1_database.pushover b8714aa2-58ce-40c7-873f-b8b94e2d53c3`
Expected: `Import successful!`

- [ ] **Step 4: R2 Bucket 생성**

Run: `cd infrastructure && tofu apply -target=cloudflare_r2_bucket.terraform_state`
Expected: `Apply complete!` (Resources: 1 added)

- [ ] **Step 5: backend.tf 복원**

```bash
cd infrastructure
mv backend.tf.bak backend.tf
```

Run: `ls infrastructure/backend.tf`
Expected: `infrastructure/backend.tf`

- [ ] **Step 6: R2 backend로 마이그레이션**

Run: `cd infrastructure && tofu init -reconfigure`
Expected: 프롬프트에서 `yes` 입력 → `Successfully configured the backend "s3"!`

---

## Task 7: Cron Trigger 리소스 생성

**Files:**
- None (runtime)

- [ ] **Step 1: Terraform plan으로 변경사항 확인**

Run: `cd infrastructure && tofu plan`
Expected: `Plan: 2 to add, 0 to change, 0 to destroy.` (R2 bucket already exists in state)

- [ ] **Step 2: Cron Trigger 생성**

Run: `cd infrastructure && tofu apply`
Expected: `Apply complete!` (Resources: 1 added - cron trigger)

- [ ] **Step 3: Cron Trigger 생성 확인**

Run: `cd infrastructure && tofu state list`
Expected: 목록에 `cloudflare_workers_cron_trigger.recovery` 포함

---

## Task 8: wrangler.toml에서 Cron 설정 제거

**Files:**
- Modify: `crates/worker/wrangler.toml`

- [ ] **Step 1: wrangler.toml에서 triggers 섹션 제거**

**Before:**
```toml
# Scheduled events (cron triggers)
[triggers]
crons = ["*/5 * * * *"]
```

**After:**
```toml
# Cron Triggers are managed by Terraform
# See: infrastructure/main.tf - cloudflare_workers_cron_trigger.recovery
```

- [ ] **Step 2: wrangler.toml 수정 확인**

Run: `grep -c "crons" crates/worker/wrangler.toml`
Expected: `0` (crons 설정이 제거됨)

---

## Task 9: Worker 재배포 및 검증

**Files:**
- None (runtime)

- [ ] **Step 1: Worker 재배포**

Run: `cd crates/worker && wrangler deploy`
Expected: `Published pushover-worker (production)`

- [ ] **Step 2: Cron Trigger 동작 확인**

Cloudflare Dashboard → Workers & Pages → pushover-worker → Triggers 탭
Expected: Cron Trigger가 `*/5 * * * *`로 표시됨

- [ ] **Step 3: 5분 후 실행 로그 확인**

Run: `wrangler tail --format pretty`
(5분 대기 후 로그 확인)
Expected: `scheduled` 이벤트 로그 표시

---

## Task 10: 변경사항 커밋

**Files:**
- None (git)

- [ ] **Step 1: 변경사항 스테이징**

```bash
git add infrastructure/backend.tf
git add infrastructure/main.tf
git add infrastructure/variables.tf
git add infrastructure/outputs.tf
git add crates/worker/wrangler.toml
```

- [ ] **Step 2: 커밋**

```bash
git commit -m "$(cat <<'EOF'
feat: Terraform으로 Cron Trigger 마이그레이션

- R2 Backend 설정 추가 (terraform state 저장용)
- cloudflare_workers_cron_trigger 리소스 추가
- wrangler.toml에서 cron 설정 제거 (Terraform 관리)
- outputs.tf에 cron_trigger_schedules 출력 추가

인프라는 Terraform, 코드 배포는 wrangler로 관리하는
하이브리드 접근 방식 적용
EOF
)"
```

- [ ] **Step 3: 커밋 확인**

Run: `git log --oneline -1`
Expected: `feat: Terraform으로 Cron Trigger 마이그레이션`

---

## Verification Checklist

- [ ] `tofu state list`에 모든 리소스 표시
- [ ] `tofu output`에서 d1_database_id, kv_namespace_id, cron_trigger_schedules 확인
- [ ] Cloudflare Dashboard에서 Cron Trigger `*/5 * * * *` 표시
- [ ] 5분 후 Worker scheduled 이벤트 실행 로그 확인
- [ ] `wrangler deploy` 정상 동작

---

## Rollback Procedure

문제 발생 시:

```bash
# 1. Cron Trigger 삭제
cd infrastructure
tofu destroy -target=cloudflare_workers_cron_trigger.recovery

# 2. wrangler.toml 원복
git checkout crates/worker/wrangler.toml

# 3. Worker 재배포
cd crates/worker && wrangler deploy
```

---

## References

- Spec: `docs/superpowers/specs/2026-03-27-opentofu-migration-design.md`
- Cloudflare Terraform Provider v4: https://registry.terraform.io/providers/cloudflare/cloudflare/4.x/docs
- workers_cron_trigger: https://registry.terraform.io/providers/cloudflare/cloudflare/latest/docs/resources/workers_cron_trigger
