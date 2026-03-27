# OpenTofu 마이그레이션 설계서

**날짜:** 2026-03-27
**상태:** Draft (v2 - 리뷰 피드백 반영)
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

### 1.4 MVP 범위 조정

> ⚠️ **현재 프로젝트 현황 분석 결과**
> - Queue는 현재 Worker 코드에서 미사용 (모든 처리가 동기적)
> - 핵심 기능(메시지 전송, D1 저장)이 TODO 상태
> - 유일하게 구현된 기능: Cron Trigger (recovery.rs)

**결정:** Cron Trigger만 1단계 마이그레이션, Queue는 MVP 완성 후 2단계 진행

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
│  - Cron Trigger (1단계)                                 │
│  - Queue (2단계 - MVP 후)                               │
│  - Queue Consumer (2단계 - MVP 후)                      │
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

#### 3.1.3 Queue (2단계 - MVP 후)

```hcl
resource "cloudflare_queue" "messages" {
  account_id = var.account_id
  name       = "pushover-messages-queue"
}
```

#### 3.1.4 Queue Consumer (2단계 - MVP 후)

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

#### 3.1.5 Cron Trigger (1단계)

```hcl
# ⚠️ 주의: cloudflare_worker_cron_trigger가 아닌
# cloudflare_workers_cron_trigger 사용 (복수형)
resource "cloudflare_workers_cron_trigger" "recovery" {
  account_id  = var.account_id
  script_name = var.worker_name

  # schedules는 object list 형식
  schedules = [
    {
      cron = "*/5 * * * *"
    }
  ]
}
```

### 3.2 Outputs

```hcl
# outputs.tf
output "d1_database_id" {
  description = "D1 Database ID (UUID)"
  value       = cloudflare_d1_database.pushover.id
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
  description = "Queue Name (2단계)"
  value       = try(cloudflare_queue.messages.name, null)
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

**After (1단계 - Cron만 마이그레이션):**
```toml
name = "pushover-worker"
compatibility_date = "2024-01-01"
account_id = "e0924c382d21ac0f10aee606b82687ce"
main = "build/worker/shim.mjs"

[[d1_databases]]
binding = "DB"
database_name = "pushover-db"
database_id = "b8714aa2-58ce-40c7-873f-b8b94e2d53c3"

# KV (현재 미사용, 필요시 추가)
# [[kv_namespaces]]
# binding = "CACHE"
# id = "<kv_namespace_id>"

# Queue (2단계에서 Terraform으로 이관 후 유지)
[[queues.producers]]
binding = "MESSAGE_QUEUE"
queue = "pushover-messages-queue"

# ⚠️ Cron 설정 제거 (Terraform 관리)
# [triggers]
# crons = ["*/5 * * * *"]
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

1. **R2 API 토큰 생성** (Terraform State용)
   - Cloudflare Dashboard → R2 → Manage R2 API Tokens
   - 권한: Object Read & Write

2. **기존 리소스 확인**
   ```bash
   wrangler queues list
   wrangler d1 list
   wrangler deployments list
   ```

3. **D1 데이터 백업** (필수!)
   ```bash
   wrangler d1 export pushover-db --output=backup_$(date +%Y%m%d_%H%M%S).sql
   ```

### 5.2 실행 순서 (1단계 - Cron Trigger)

```bash
# 0. backend.tf 임시 비활성화 (순환 의존 해결)
cd infrastructure
mv backend.tf backend.tf.bak

# 1. 로컬 state로 초기화
tofu init -backend=false

# 2. R2 버킷 생성 (state 저장용)
tofu apply -target=cloudflare_r2_bucket.terraform_state

# 3. backend.tf 복원
mv backend.tf.bak backend.tf

# 4. R2 backend로 마이그레이션
tofu init -reconfigure
# → "Do you want to copy existing state?" → yes

# 5. 기존 리소스 import
tofu import cloudflare_d1_database.pushover <database_id>
tofu import cloudflare_workers_kv_namespace.cache <namespace_id>

# 6. Cron Trigger 생성 (새 리소스)
tofu apply

# 7. wrangler.toml 수정 (cron 섹션 제거)
cd ../crates/worker
# wrangler.toml 편집

# 8. Worker 재배포
wrangler deploy

# 9. Cron Trigger 동작 확인
wrangler deployments list
# 5분 후 Cloudflare Dashboard에서 cron 실행 확인
```

### 5.3 실행 순서 (2단계 - Queue, Queue Consumer)

> MVP 완성 후 진행

```bash
# 1. Queue import
tofu import cloudflare_queue.messages <queue_id>

# 2. Queue Consumer 생성
tofu apply

# 3. wrangler.toml에서 Consumer 설정 제거
```

### 5.4 롤백 계획

#### 5.4.1 사전 백업 (필수)

```bash
# D1 데이터 백업
wrangler d1 export pushover-db --output=backup_$(date +%Y%m%d).sql

# KV 데이터 백업 (사용 중인 경우)
wrangler kv:key list --namespace-id=<kv_id> > kv_keys_backup.json

# 현재 wrangler.toml 백업
cp wrangler.toml wrangler.toml.backup
```

#### 5.4.2 롤백 절차

```bash
# 1. Terraform 리소스 중 Cron Trigger만 삭제
tofu destroy -target=cloudflare_workers_cron_trigger.recovery

# ⚠️ D1, KV는 삭제하지 않음 (데이터 보존)
# tofu destroy -target=cloudflare_d1_database.pushover  # 절대 금지!

# 2. wrangler.toml 원복
cp wrangler.toml.backup wrangler.toml

# 3. Worker 재배포 (Cron을 wrangler로 복구)
wrangler deploy

# 4. D1 데이터 복구 (필요시)
wrangler d1 execute pushover-db --file=backup_YYYYMMDD.sql
```

#### 5.4.3 재해 복구 시나리오

| 시나리오 | 복구 방법 |
|----------|-----------|
| Cron Trigger 생성 실패 | wrangler.toml cron 섹션 복구 후 deploy |
| D1 연결 실패 | database_id 확인, wrangler.toml 바인딩 점검 |
| State 파일 손상 | 로컬 백업에서 복구 또는 `tofu import` 재실행 |
| 전체 롤백 | 5.4.2 절차 실행 |

---

## 6. 제약사항

### 6.1 Cloudflare Terraform Provider v4 한계

- `cloudflare_queue_consumer` 리소스는 v4.x에서 실험적 기능
- `cloudflare_queue`의 `producers` 속성은 **read-only** → Producer 생성은 wrangler 전용
- `cloudflare_workers_cron_trigger` 사용 (단수형 `cloudflare_worker_cron_trigger` 아님)

### 6.2 환경 분리

현재 단일 환경(dev)만 지원. 추후 환경 분리 필요 시:
- Terraform Workspace 사용
- 또는 디렉토리 분리 (`infrastructure/dev/`, `infrastructure/prod/`)

### 6.3 R2 Backend 순환 의존

R2 버킷을 Terraform으로 생성하면서 그 버킷을 backend로 사용하는 순환 문제:
- **해결:** 초기에는 로컬 state 사용, R2 버킷 생성 후 `-reconfigure`로 마이그레이션

---

## 7. 성공 기준

### 7.1 1단계 (Cron Trigger)

- [ ] `tofu apply` 성공
- [ ] `cloudflare_workers_cron_trigger.recovery` 리소스 생성 확인
- [ ] 5분 후 Cloudflare Dashboard에서 cron 실행 로그 확인
- [ ] 기존 Worker 기능 정상 동작
- [ ] `wrangler deploy` 후 바인딩 정상 연결

### 7.2 2단계 (Queue - MVP 후)

- [ ] `cloudflare_queue.messages` import 성공
- [ ] `cloudflare_queue_consumer.pushover` 생성 성공
- [ ] Queue 메시지 처리 정상 동작

---

## 8. 참고 자료

- [Cloudflare Terraform Provider v4 문서](https://registry.terraform.io/providers/cloudflare/cloudflare/4.x/docs)
- [Cloudflare Queues Terraform](https://developers.cloudflare.com/queues/)
- [Terraform S3 Backend with R2](https://developers.cloudflare.com/r2/api/s3/api/)
- [cloudflare_workers_cron_trigger 스키마](https://registry.terraform.io/providers/cloudflare/cloudflare/latest/docs/resources/workers_cron_trigger)

---

## 9. 변경 이력

| 버전 | 날짜 | 변경 내용 |
|------|------|-----------|
| v1 | 2026-03-27 | 초기 작성 |
| v2 | 2026-03-27 | 리뷰 피드백 반영: Cron Trigger 스키마 수정, D1 output 속성 수정, R2 backend 순환 문제 해결, 롤백 계획 보강, MVP 범위 조정 |
