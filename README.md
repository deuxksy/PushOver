# PushOver Serverless Platform

PushOver API를 위한 Rust 와 TypeScript 기반의 Cloudflare Serverless 알림 시스템입니다.

## 목차

- [아키텍처](#-아키텍처)
- [Cloudflare 서비스](#-cloudflare)
- [프로젝트 구조](#-프로젝트-구조)
- [데이터베이스](#-데이터베이스)
- [개발 시나리오](#-개발-시나리오)

---

## 🏗️ 아키텍처

### 전체 시스템 아키텍처

```mermaid
graph TB
    subgraph Clients
        CLI[CLI<br/>Rust]
        DASH[Dashboard<br/>Next.js]
        EXT[External Services]
    end

    subgraph "Cloudflare Worker (단일 Worker)"
        direction TB
        HANDLER[HTTP Handler<br/>API Routes]
        CONSUMER[Queue Consumer<br/>메시지 처리]
        CRON[Cron Handler<br/>*/5분 복구]
    end

    subgraph "Cloudflare Storage"
        QUEUE[(Queue<br/>메시지 큐)]
        KV[(KV<br/>Token 캐시/실패 백업)]
        D1[(D1 Database<br/>메시지/웹훅)]
        R2[(R2<br/>이미지/D1 백업)]
    end

    subgraph "External APIs"
        PO[PushOver API]
    end

    subgraph "CI/CD"
        GHA[GitHub Actions<br/>D1 Backup<br/>매일 18:00 UTC]
    end

    CLI -->|POST /messages| HANDLER
    CLI -->|POST /tokens/register| HANDLER
    DASH -->|GET /messages| HANDLER
    DASH -->|GET/POST/DELETE /webhooks| HANDLER
    EXT -->|POST /webhooks<br/>Callback| HANDLER

    HANDLER -->|Token 검증| KV
    HANDLER -->|이미지 업로드| R2
    HANDLER -->|메시지 투입| QUEUE
    HANDLER --> D1
    QUEUE -->|메시지 소비| CONSUMER
    CONSUMER -->|Send Message| PO
    CONSUMER -->|성공: 저장| D1
    CONSUMER -->|실패: 백업| KV
    CRON -->|failed 메시지 복구| CONSUMER
    GHA -->|D1 Full Export → R2| R2
    PO -->|Delivery Callback| HANDLER

    style HANDLER fill:#f38020,color:#fff
    style CONSUMER fill:#f38020,color:#fff
    style CRON fill:#f38020,color:#fff
    style GHA fill:#2088ff,color:#fff
    style QUEUE fill:#f5a623,color:#fff
    style KV fill:#f5a623,color:#fff
    style D1 fill:#f5a623,color:#fff
    style R2 fill:#f5a623,color:#fff
```

### 메시지 전송 흐름 (Queue-First)

```mermaid
sequenceDiagram
    participant C as Client
    participant W as Worker (HTTP Handler)
    participant KV as KV (Token 캐시)
    participant Q as Queue
    participant R2 as R2 Storage
    participant D1 as D1 Database
    participant CON as Worker (Queue Consumer)
    participant PO as PushOver API

    C->>W: POST /api/v1/messages
    W->>KV: Token 검증 (캐시 조회)
    alt KV hit
        KV-->>W: user_key
    else KV miss
        W->>D1: Token 조회
        D1-->>W: user_key
        W->>KV: 캐시 저장 (TTL 1h)
    end
    opt 이미지 첨부 시
        W->>R2: 이미지 업로드
        R2-->>W: 이미지 URL
    end
    W->>Q: 메시지 투입
    W-->>C: 202 Accepted {status: queued}

    Q->>CON: 메시지 소비
    CON->>PO: PushOver API 호출
    alt 성공
        PO-->>CON: 200 OK {receipt}
        CON->>D1: 메시지 저장 (status=sent)
    else 실패
        PO-->>CON: Error
        CON->>KV: 실패 메시지 백업 (TTL 7d)
        CON->>D1: failed_deliveries 기록
    end
```

### 재시도 메커니즘 (Cron + KV)

```mermaid
flowchart TD
    A[Queue Consumer: 전송 실패] --> B["KV에 메시지 백업<br/>pushover-failed:id<br/>TTL 7d"]
    B --> C["D1 failed_deliveries 기록<br/>retry_count++"]
    C --> D{"retry_count < 3?"}

    D -->|Yes| E["Cron Handler<br/>*/5분 실행"]
    E --> F[KV에서 메시지 본문 복원]
    F --> G[PushOver API 재전송]

    D -->|No| H[최종 실패<br/>KV TTL 만료로 자동 정리]

    G --> I{전송 성공?}
    I -->|Yes| J[D1 status=sent 업데이트<br/>KV 백업 키 삭제]
    I -->|No| D

    style A fill:#ff6b6b
    style J fill:#51cf66
    style H fill:#ff6b6b
```

### 웹훅 처리 흐름

```mermaid
sequenceDiagram
    participant PO as PushOver API
    participant W as Worker API
    participant D1 as D1 Database
    participant EXT as 등록된 Webhook

    PO->>W: POST /api/v1/webhooks (Callback)
    Note over PO,W: X-Pushover-Signature 헤더 포함

    W->>W: HMAC-SHA256 서명 검증
    alt 서명 불일치
        W-->>PO: 401 Unauthorized
    end

    W->>D1: 메시지 상태 업데이트 (delivered/acknowledged)
    W->>EXT: 등록된 Webhook에 이벤트 전달
    W->>D1: webhook_deliveries 기록
    W-->>PO: 200 OK
```

### 웹훅 서명 검증

```mermaid
flowchart LR
    A[웹훅 요청] --> B{X-Pushover-Signature<br/>헤더 존재?}
    B -->|No| C[401 Unauthorized]
    B -->|Yes| D[HMAC-SHA256<br/>서명 계산]
    D --> E{Timing-safe<br/>비교}
    E -->|불일치| C
    E -->|일치| F[요청 처리]

    style C fill:#ff6b6b
    style F fill:#51cf66
```

---

## ☁️ Cloudflare

| 서비스 | 용도 | 관리 도구 |
| -------- | ------ | ---------- |
| **Workers** | Serverless API 서버 (Rust/WASM) | Wrangler |
| **Pages** | 정적 호스팅 (Dashboard) | Wrangler |
| **Queues** | 비동기 메시지 큐 (Producer-Consumer) | OpenTofu |
| **KV** | Token 캐시, Webhook 캐시, 실패 메시지 백업 | OpenTofu |
| **D1** | SQLite 기반 DB (스키마: [`migrations/`](./migrations/)) | OpenTofu |
| **R2** | 오브젝트 스토리지 (Terraform state, D1 백업, 메시지 이미지) | OpenTofu |
| **Cron Triggers** | 스케줄러 (Recovery Worker, */5분) | OpenTofu |

### 삭제 및 복구 절차

```mermaid
flowchart LR
    A[① 백업<br/>make db-backup] --> B[② 삭제<br/>make destroy-all]
    B --> C[③ 재생성<br/>make init/plan/apply]
    C --> D[④ 복구<br/>make db-restore]
    D --> E[⑤ 배포<br/>make deploy]

    style A fill:#ff6b6b,color:#fff
    style B fill:#ff6b6b,color:#fff
    style E fill:#51cf66,color:#fff
```

| # | 단계 | 명령 | 설명 |
| - | ---- | ---- | ---- |
| ① | 백업 | `make db-backup` | D1 → 로컬 SQL dump (삭제 전 필수) |
| ② | 삭제 | `make destroy-all` | Pages,Worker → 인프라(D1, KV, R2, Queues, Cron) 순서대로 전체 삭제 |
| ③ | 재생성 | `make init && make plan && make apply` | OpenTofu로 인프라 재생성 |
| ④ | 복구 | `make db-restore file=backups/d1-xxx.sql` | SQL dump(스키마+데이터) → D1 복구 |
| ⑤ | 배포 | `make deploy` | Worker + Pages 재배포 |

---

## 📦 프로젝트 구조

```bash
pushover/
├── crates/
│   ├── sdk/                    # Rust SDK
│   ├── cli/                    # CLI 도구
│   └── worker/                 # Cloudflare Worker
│       └── wrangler.toml
├── dashboard/                  # Next.js 웹 UI
├── migrations/                  # D1 마이그레이션
├── infrastructure/              # OpenTofu 인프라
└── docs/                       # 문서
```

---

## 🗄️ 데이터베이스

### 스키마

> 상세: [`migrations/`](./migrations/) SQL 파일 참조

**D1 테이블**:

- `api_tokens` - API 인증 토큰
- `messages` - 메시지 전송 기록
- `webhooks` - 웹훅 등록 정보
- `webhook_deliveries` - 웹훅 전송 기록
- `failed_deliveries` - 실패한 메시지 (재시도용)

### 백업

```mermaid
flowchart LR
    CRON[Cron<br/>매일 18:00 UTC] --> GHA[GitHub Actions]
    GHA -->|wrangler d1 export| SQL[SQL Dump]
    SQL -->|wrangler r2 object put| R2[(R2 Bucket<br/>pushover-backups)]
    R2 -->|7일 초과| DEL[R2 수명 주기 규칙<br/>자동 삭제]

    style GHA fill:#2088ff,color:#fff
    style R2 fill:#f5a623,color:#fff
```

| 항목 | 내용 |
| ------ | ------ |
| **워크플로우** | `.github/workflows/d1-backup.yml` |
| **주기** | 매일 18:00 UTC (한국 03:00) + `workflow_dispatch` 수동 실행 |
| **방식** | `wrangler d1 export` → 전체 SQL dump |
| **저장소** | R2 `pushover-backups/d1-full-backup/` |
| **보존** | 7일 (R2 버킷 수명 주기 규칙으로 자동 삭제) |

### 복구

| 항목 | 내용 |
| ------ | ------ |
| **방식** | `wrangler d1 execute --file=backup.sql` |
| **로컬** | `make db-restore-local file=backups/xxx.sql` |
| **원격** | `make db-restore file=backups/xxx.sql` |

> **참고**: `tofu destroy`로 R2 버킷도 삭제되면 R2 내 D1 백업도 함께 삭제됩니다. 전체 삭제 전 반드시 `make db-backup`으로 로컬에 SQL dump를 확보하세요.

---

## 🚀 개발 시나리오

모든 명령은 `Makefile` 타겟으로 관리됩니다. 각 타겟의 상세 내용은 `Makefile` 주석을 참조하세요.

### 사전 요구사항

- Rust >= 1.92.0
- Node.js >= v24.14.0

### 환경변수

```bash
cp .env.example .env
# .env 파일을 실제 값으로 변경
```

| 변수명 | 발급처 |
| -------- | -------- |
| `CLOUDFLARE_API_TOKEN` | [Cloudflare Dashboard](https://dash.cloudflare.com/profile/api-tokens) |
| `CLOUDFLARE_ACCOUNT_ID` | Cloudflare Dashboard 사이드바 |
| `PUSHOVER_USER_KEY` | [PushOver](https://pushover.net) |
| `PUSHOVER_API_TOKEN` | PushOver Settings → Applications |
| `PUSHOVER_WEBHOOK_SECRET` | `openssl rand -base64 32` |

### 개발 프로세스

```mermaid
graph LR
    A[① 인프라<br/>make init/plan/apply] --> B[② 마이그레이션<br/>make migrate]
    B --> C[③ 셋업<br/>make setup]
    C --> D[④ 정리<br/>make clean]
    D --> E[⑤ 빌드<br/>make build]
    E --> F[⑥ 정적 분석<br/>make check/lint]
    E --> G[⑦ 배포<br/>make deploy]
    G --> H[⑧ 테스트<br/>make test]
    E --> I[⑨ 로컬 개발<br/>make dev]
    G --> J[⑩ 백업<br/>make db-backup]
    J --> K[⑪ 복구<br/>make db-restore]

    style A fill:#f38020,color:#fff
    style E fill:#f38020,color:#fff
    style G fill:#f38020,color:#fff
    style J fill:#2088ff,color:#fff
    style K fill:#2088ff,color:#fff
```

| # | 단계 | make 타겟 | 설명 |
|---|------|-----------|------|
| ① | 인프라 | `make init && make plan && make apply` | OpenTofu로 D1, KV, R2, Cron 생성 |
| ② | 마이그레이션 | `make migrate` | D1 스키마 적용 |
| ③ | 셋업 | `make setup` | pnpm install + Rust workspace check |
| ④ | 정리 | `make clean` | 빌드 산출물 전체 삭제 |
| ⑤ | 빌드 | `make build` | Dashboard (Next.js) + Worker (WASM) |
| ⑥ | 정적 분석 | `make check` / `make lint` | 타입 검사 / clippy 린트 |
| ⑦ | 배포 | `make deploy` | Cloudflare Pages + Workers 배포 |
| ⑧ | 테스트 | `make test` | SDK → CLI → Worker → Dashboard |
| ⑨ | 로컬 개발 | `make dev` | wrangler dev + Next.js dev 서버 |
| ⑩ | 백업 | `make db-backup` | D1 전체 SQL dump |
| ⑪ | 복구 | `make db-restore file=...` | SQL dump → D1 복구 |

---

## 📝 라이선스

MIT
