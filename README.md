# PushOver Serverless Platform

PushOver API를 위한 Rust 와 TypeScript 기반의 Cloudflare Serverless 알림 시스템입니다.

## 목차

- [아키텍처](#-아키텍처)
- [Cloudflare 서비스](#-cloudflare-서비스)
- [프로젝트 구조](#-프로젝트-구조)
- [데이터베이스](#-데이터베이스)
- [빠른 시작](#-빠른-시작)
- [테스트](#-테스트)
- [배포](#-배포)

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

    subgraph "Cloudflare Edge"
        WORKER[Worker API<br/>Rust/WASM]
        QUEUE[Queue<br/>메시지 큐]
        CONSUMER[Consumer<br/>Worker]
        KV[(KV<br/>캐시/백업)]
        D1[(D1 Database)]
        R2[(R2<br/>이미지/백업)]
        CRON[Cron Trigger<br/>*/5분]
    end

    subgraph "External APIs"
        PO[PushOver API]
    end

    CLI -->|POST /messages| WORKER
    CLI -->|POST /tokens/register| WORKER
    DASH -->|GET /messages| WORKER
    DASH -->|GET/POST/DELETE /webhooks| WORKER
    EXT -->|POST /webhooks<br/>Callback| WORKER
    CRON -->|handle_failed_messages| CONSUMER

    WORKER -->|Token 검증| KV
    WORKER -->|이미지 업로드| R2
    WORKER -->|메시지 투입| QUEUE
    WORKER --> D1
    QUEUE -->|메시지 소비| CONSUMER
    CONSUMER -->|Send Message| PO
    CONSUMER -->|성공: 저장| D1
    CONSUMER -->|실패: 백업| KV
    CRON -->|D1 백업 스냅샷| R2
    PO -->|Delivery Callback| WORKER

    style WORKER fill:#f38020,color:#fff
    style QUEUE fill:#f38020,color:#fff
    style CONSUMER fill:#f38020,color:#fff
    style KV fill:#f38020,color:#fff
    style D1 fill:#f38020,color:#fff
    style R2 fill:#f38020,color:#fff
    style CRON fill:#f38020,color:#fff
```

### 메시지 전송 흐름 (Queue-First)

```mermaid
sequenceDiagram
    participant C as Client
    participant W as Worker API
    participant KV as KV (Token 캐시)
    participant Q as Queue
    participant R2 as R2 Storage
    participant D1 as D1 Database
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

### 재시도 메커니즘 (KV 기반)

```mermaid
flowchart TD
    A[메시지 전송 실패] --> B[KV에 메시지 본문 백업<br/>TTL 7d]
    B --> C[D1 failed_deliveries에 기록]
    C --> D{재시도 횟수 < 3?}

    D -->|Yes| E[Cron Trigger<br/>*/5분]
    E --> F[KV에서 메시지 본문 복원]
    F --> G[PushOver API 재전송]

    D -->|No| H[최종 실패<br/>KV TTL 만료로 자동 정리]

    G --> I{전송 성공?}
    I -->|Yes| J[D1 상태 업데이트<br/>status=sent<br/>KV 키 삭제]
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

## ☁️ Cloudflare 서비스

| 서비스 | 용도 |
| -------- | ------ |
| **Workers** | Serverless API 서버 (Rust/WASM) |
| **Queues** | 비동기 메시지 큐 (Producer-Consumer) |
| **KV** | Token 캐시, Webhook 캐시, 실패 메시지 백업 |
| **D1** | SQLite 기반 DB (스키마: [`migrations/`](./migrations/)) |
| **Pages** | 정적 호스팅 (Dashboard) |
| **R2** | 오브젝트 스토리지 (Terraform state, D1 백업, 메시지 이미지) |
| **Cron Triggers** | 스케줄러 (Recovery Worker, */5분) |

> 인프라 관리: D1, KV, R2, Cron Trigger는 `infrastructure/`의 **OpenTofu**로 관리. Worker 배포는 `wrangler` 사용.

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

> 스키마 상세: [`migrations/`](./migrations/) SQL 파일 참조
>
**D1 테이블**:

- `api_tokens` - API 인증 토큰
- `messages` - 메시지 전송 기록
- `webhooks` - 웹훅 등록 정보
- `webhook_deliveries` - 웹훅 전송 기록
- `failed_deliveries` - 실패한 메시지 (재시도용)

---

## 🚀 빠른 시작

### 사전 요구사항

**개발 환경**:

- Rust 1.92.0
- Node.js v24.14.0
- pnpm 10.30.3

```bash
# Cloudflare Workers Rust/WASM 빌드 도구
cargo install worker-build
```

### 환경변수 설정

```bash
cp .env.example .env
# .env 파일을 실제 값으로 변경
```

**필수 환경변수**:

| 변수명 | 발급처 |
| -------- | -------- |
| `CLOUDFLARE_API_TOKEN` | [Cloudflare Dashboard](https://dash.cloudflare.com/profile/api-tokens) |
| `CLOUDFLARE_ACCOUNT_ID` | Cloudflare Dashboard 사이드바 |
| `PUSHOVER_USER_KEY` | [PushOver](https://pushover.net) |
| `PUSHOVER_API_TOKEN` | PushOver Settings → Applications |
| `PUSHOVER_WEBHOOK_SECRET` | `openssl rand -base64 32` |

### 설치 및 실행

```bash
# 의존성 설치
pnpm install

# Worker 빌드 & 배포
cd crates/worker
wrangler deploy

# Dashboard 실행
cd dashboard && pnpm dev
```

---

## 🧪 테스트

### Worker API

```bash
make test-api           # 기본 실행
make test-api-verbose   # 상세 출력
```

### Dashboard (Playwright)

```bash
make dashboard-test-loc   # 로컬 테스트
make dashboard-test-dev   # dev 환경 테스트
make dashboard-test-all   # 전체 테스트
```

---

## 🚀 배포

### Infrastructure (OpenTofu)

```bash
cd infrastructure
tofu init
tofu plan
tofu apply
```

### Worker

```bash
cd crates/worker
wrangler deploy
```

### Dashboard (Cloudflare Pages)

```bash
cd dashboard
pnpm run pages:build
pnpm run deploy
```

---

## 📝 라이선스

MIT
