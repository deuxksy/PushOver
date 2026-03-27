# OpenTofu 마이그레이션 설계서

**날짜:** 2026-03-27
**상태:** Draft
**작성자:** Claude

---

## 1. 개요

### 1.1 목표

PushOver Serverless Platform의 Cloudflare 인프라를 wrangler에서 OpenTofu(Terraform)로 완전히 마이그레이션합니다.

### 1.2 현재 상태

| 리소스 | 관리 도구 | 마이그레이션 필요 |
|--------|-----------|------------------|
| D1 Database | Terraform | ❌ |
| KV Namespace | Terraform | ❌ |
| Queue | wrangler | ✅ |
| Queue Consumer | wrangler | ✅ |
| Cron Trigger | wrangler | ✅ |
| Worker Script | wrangler | ❌ (코드는 wrangler 유지) |

### 1.3 목표 상태

- **Terraform 관리:** D1, KV, Queue, Queue Consumer, Cron Trigger, R2 (state)
- **wrangler 관리:** Worker Script 코드 배포만

---

## 2. 아키텍처

### 2.1 책임 분리

```
┌─────────────────────────────────────────────────────────┐
│                    OpenTofu (Terraform)                  │
│                                                          │
│  인프라 리소스 생성 및 관리                               │
│  - D1 Database                                          │
│  - KV Namespace                                         │
│  - Queue                                                │
│  - Queue Consumer                                       │
│  - Cron Trigger                                         │
│  - R2 Bucket (state 저장용)                             │
│                                                          │
│  Outputs 제공 → wrangler에서 참조                        │
└─────────────────────────────────────────────────────────┘
                           │
                           │ outputs (database_id, queue_name, etc.)
                           ▼
┌─────────────────────────────────────────────────────────┐
│                    wrangler                              │
│                                                          │
│  Worker 코드 배포만 담당                                 │
│  - Rust/WASM 빌드                                       │
│  - Worker Script 업로드                                 │
│  - bindings은 Terraform outputs 또는 하드코딩 참조      │
└─────────────────────────────────────────────────────────┘
```

### 2.2 하이브리드 접근 이유

1. **Cloudflare Terraform Provider v5 불안정성**
   - `cloudflare_workers_script` 리소스에 다수의 이슈 존재
   - 공식 문서에서도 v5 마이그레이션 보류 권장

2. **빌드 산출물 관리**
   - Rust/WASM 빌드 결과물을 git에 포함하면 repo 크기 증가
   - wrangler는 빌드 후 즉시 배포 가능

3. **개발 편의성**
   - 로컬 개발 시 `wrangler dev` 사용 가능
   - 코드 변경 시 terraform apply 불필요

---

## 3. 리소스 상세 설계

### 3.1 Terraform 리소스

#### 3.1.1 Backend (R2)

```hcl
# backend.tf
terraform {
  backend "s3" {
    bucket = "pushover-terraform-state"
    key    = "terraform.tfstate"
    region = "auto"

    endpoints = {
      s3 = "https://<account_id>.r2.cloudflarestorage.com"
    }

    skip_credentials_validation = true
    skip_metadata_api_check     = true
    skip_region_validation      = true
    skip_requesting_account_id  = true
  }
}
```

#### 3.1.2 R2 Bucket (State 저장용)

```hcl
resource "cloudflare_r2_bucket" "terraform_state" {
  account_id = var.account_id
  name       = "pushover-terraform-state"
  location   = "auto"
}
```

#### 3.1.3 Queue

```hcl
resource "cloudflare_queue" "messages" {
  account_id = var.account_id
  name       = "pushover-messages-queue"
}
```

#### 3.1.4 Queue Consumer

```hcl
resource "cloudflare_queue_consumer" "pushover" {
  account_id        = var.account_id
  queue_id          = cloudflare_queue.messages.id
  script_name       = var.worker_name

  # 기본 설정
  settings {
    batch_size = 10
    max_retries = 3
    max_wait_time_ms = 5000
  }
}
```

#### 3.1.5 Cron Trigger

```hcl
resource "cloudflare_worker_cron_trigger" "recovery" {
  account_id  = var.account_id
  script_name = var.worker_name
  schedules   = ["*/5 * * * *"]
}
```

### 3.2 Outputs

```hcl
# outputs.tf
output "d1_database_id" {
  description = "D1 Database ID"
  value       = cloudflare_d1_database.pushover.database_id
}

output "d1_database_name" {
  description = "D1 Database Name"
  value       = cloudflare_d1_database.pushover.name
}

output "kv_namespace_id" {
  description = "KV Namespace ID"
  value       = cloudflare_workers_kv_namespace.cache.id
}

output "queue_name" {
  description = "Queue Name"
  value       = cloudflare_queue.messages.name
}

output "worker_script_name" {
  description = "Worker Script Name"
  value       = var.worker_name
}
```

### 3.3 wrangler.toml 변경

**Before (현재):**
```toml
name = "pushover-worker"
compatibility_date = "2024-01-01"
account_id = "e0924c382d21ac0f10aee606b82687ce"
main = "build/worker/shim.mjs"

[[d1_databases]]
binding = "DB"
database_name = "pushover-db"
database_id = "b8714aa2-58ce-40c7-873f-b8b94e2d53c3"

[[queues.producers]]
binding = "MESSAGE_QUEUE"
queue = "pushover-messages-queue"

[[queues.consumers]]
queue = "pushover-messages-queue"
max_batch_size = 10

[triggers]
crons = ["*/5 * * * *"]
```

**After (변경):**
```toml
name = "pushover-worker"
compatibility_date = "2024-01-01"
account_id = "e0924c382d21ac0f10aee606b82687ce"
main = "build/worker/shim.mjs"

# D1 - 하드코딩 또는 terraform output에서 참조
[[d1_databases]]
binding = "DB"
database_name = "pushover-db"
database_id = "b8714aa2-58ce-40c7-873f-b8b94e2d53c3"

# KV - 하드코딩 또는 terraform output에서 참조
[[kv_namespaces]]
binding = "CACHE"
id = "<kv_namespace_id>"

# Queue Producer만 유지 (Consumer는 Terraform 관리)
[[queues.producers]]
binding = "MESSAGE_QUEUE"
queue = "pushover-messages-queue"

# Consumer, Cron 설정 제거 (Terraform 관리)
```

---

## 4. 파일 구조

```
infrastructure/
├── backend.tf           # R2 backend 설정
├── main.tf              # 메인 리소스 정의
├── variables.tf         # 입력 변수
├── outputs.tf           # 출력값
├── terraform.tfvars     # 변수 값 (gitignore)
└── .terraform.lock.hcl  # provider 버전 고정
```

### 4.1 변수 정의

```hcl
# variables.tf
variable "cloudflare_api_token" {
  type        = string
  description = "Cloudflare API Token"
  sensitive   = true
}

variable "account_id" {
  type        = string
  description = "Cloudflare Account ID"
}

variable "worker_name" {
  type        = string
  description = "Worker Script Name"
  default     = "pushover-worker"
}

variable "access_key_id" {
  type        = string
  description = "R2 Access Key ID for Terraform State"
  sensitive   = true
}

variable "secret_access_key" {
  type        = string
  description = "R2 Secret Access Key for Terraform State"
  sensitive   = true
}
```

---

## 5. 마이그레이션 절차

### 5.1 사전 준비

1. R2 API 토큰 생성 (Terraform State용)
2. 기존 리소스 확인
   ```bash
   wrangler queues list
   wrangler d1 list
   ```

### 5.2 실행 순서

```bash
# 1. R2 버킷 생성 (state용)
cd infrastructure
tofu init -backend=false
tofu apply -target=cloudflare_r2_bucket.terraform_state

# 2. Backend 초기화
tofu init -reconfigure

# 3. 기존 리소스 import
tofu import cloudflare_d1_database.pushover <database_id>
tofu import cloudflare_workers_kv_namespace.cache <namespace_id>
tofu import cloudflare_queue.messages <queue_id>

# 4. 새 리소스 생성
tofu apply

# 5. wrangler.toml 수정 후 재배포
cd ../crates/worker
wrangler deploy
```

### 5.3 롤백 계획

문제 발생 시:
1. Terraform 리소스 `tofu destroy`로 삭제
2. wrangler.toml 원복
3. `wrangler deploy`로 기존 방식 복구

---

## 6. 제약사항

### 6.1 Cloudflare Terraform Provider v4 한계

- `cloudflare_queue_consumer` 리소스는 v4.x에서 실험적 기능
- 안정성을 위해 `cloudflare_worker_cron_trigger` 사용 권장

### 6.2 환경 분리

현재 단일 환경(dev)만 지원. 추후 환경 분리 필요 시:
- Terraform Workspace 사용
- 또는 디렉토리 분리 (`infrastructure/dev/`, `infrastructure/prod/`)

---

## 7. 성공 기준

- [ ] `tofu apply` 성공
- [ ] Queue Consumer 정상 작동 확인
- [ ] Cron Trigger 5분마다 실행 확인
- [ ] 기존 Worker 기능 정상 동작
- [ ] `wrangler deploy` 후 바인딩 정상 연결

---

## 8. 참고 자료

- [Cloudflare Terraform Provider v4 문서](https://registry.terraform.io/providers/cloudflare/cloudflare/4.x/docs)
- [Cloudflare Queues Terraform](https://developers.cloudflare.com/queues/)
- [Terraform S3 Backend with R2](https://developers.cloudflare.com/r2/api/s3/api/)
