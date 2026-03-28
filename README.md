# PushOver Serverless Platform

PushOver API를 위한 Rust 기반 Serverless 플랫폼입니다. Cloudflare Workers + D1을 활용한 알림 시스템입니다.

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
        D1[(D1 Database)]
    end

    subgraph "External APIs"
        PO[PushOver API]
    end

    CLI -->|HTTP/REST| WORKER
    DASH -->|HTTP/REST| WORKER
    EXT -->|Webhook| WORKER

    WORKER --> D1
    WORKER -->|Send Message| PO
    PO -->|Callback| WORKER

    style WORKER fill:#f38020,color:#fff
    style D1 fill:#f38020,color:#fff
```

### 메시지 전송 흐름

```mermaid
sequenceDiagram
    participant C as Client
    participant W as Worker API
    participant D1 as D1 Database
    participant PO as PushOver API

    C->>W: POST /api/v1/messages
    W->>W: Bearer 토큰 검증 → user_key 획득
    W->>PO: PushOver API 호출
    alt 성공
        PO-->>W: 200 OK {receipt}
        W->>D1: 메시지 저장 (status=sent)
        W-->>C: {status: success, receipt}
    else 실패
        PO-->>W: Error
        W->>D1: 메시지 저장 (status=failed)
        W->>D1: failed_deliveries 기록
        W-->>C: 502 {status: error}
    end
```

### 재시도 메커니즘

```mermaid
flowchart TD
    A[메시지 전송 실패] --> B[D1 failed_deliveries에 기록]
    B --> C{재시도 횟수 < 3?}

    C -->|Yes| D[Cron Trigger<br/>Recovery Worker]
    D --> E[PushOver API 재전송]

    C -->|No| F[최종 실패]

    E --> G{전송 성공?}
    G -->|Yes| H[D1 상태 업데이트<br/>status=sent]
    G -->|No| C

    style A fill:#ff6b6b
    style H fill:#51cf66
    style F fill:#ff6b6b
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

| 서비스 | 용도 | 비고 |
| -------- | ------ |------|
| **Workers** | Serverless API 서버 | Rust/WASM으로 빌드 |
| **D1** | SQLite 기반 DB | 메시지, 웹훅, 인증 토큰 저장 |
| **Pages** | 정적 호스팅 | Dashboard 배포용 |
| **KV** | 키-값 스토어 | 캐시 및 실패 메시지 백업 |
| **R2** | 오브젝트 스토리지 | Terraform 상태 파일 저장 |
| **Cron Triggers** | 스케줄러 | Recovery Worker (실패 메시지 재시도, */5 분) |

> **인프라 관리**: D1, KV, R2, Cron Trigger는 `infrastructure/` 디렉토리의 **OpenTofu**로 관리. Worker 배포는 `wrangler` 사용.

---

## 📦 프로젝트 구조

```bash
pushover/
├── crates/
│   ├── sdk/                    # Rust SDK
│   │   ├── src/
│   │   │   ├── lib.rs           # 공개 API
│   │   │   ├── models.rs        # 데이터 모델
│   │   │   ├── error.rs         # 에러 타입
│   │   │   ├── http_client.rs  # HTTP 클라이언트
│   │   │   └── webhook.rs       # 웹훅 검증
│   │   └── tests/
│   │
│   ├── cli/                    # CLI 도구
│   │   ├── src/
│   │   │   ├── main.rs          # 진입점
│   │   │   ├── commands/
│   │   │   │   ├── send.rs      # 메시지 전송
│   │   │   │   └── history.rs   # 이력 조회
│   │   │   └── config.rs        # 설정 관리
│   │
│   └── worker/                 # Cloudflare Worker
│       ├── src/
│       │   ├── lib.rs           # 진입점 + 라우터
│       │   ├── routes.rs        # API 라우트 핸들러
│       │   ├── middleware.rs     # CORS, 인증
│       │   ├── types.rs         # 요청/응답 타입
│       │   ├── db.rs            # D1 데이터베이스 리포지토리
│       │   ├── pushover.rs      # PushOver API 클라이언트
│       │   ├── crypto.rs        # HMAC 서명 생성/검증
│       │   ├── recovery.rs      # 실패 메시지 복구
│       │   └── utils.rs         # 유틸리티
│       └── wrangler.toml
│
├── dashboard/                  # Next.js 웹 UI
│   ├── src/
│   │   ├── app/
│   │   │   ├── page.tsx         # 메인 페이지
│   │   │   ├── history/         # 이력 페이지
│   │   │   └── settings/        # 설정 페이지
│   │   └── lib/
│   │       ├── api.ts           # API 클라이언트
│   │       └── settings.ts      # 설정 관리 (localStorage)
│   ├── tests/e2e/               # Playwright E2E 테스트
│   └── package.json
│
├── migrations/                  # D1 마이그레이션
│   ├── 0001_init.sql
│   ├── 0002_add_api_token.sql
│   └── 0003_api_tokens.sql
│
├── infrastructure/              # OpenTofu 인프라
│   ├── main.tf                  # D1, KV, R2, Cron Trigger
│   ├── backend.tf               # R2 원격 상태 백엔드
│   ├── variables.tf             # 변수 정의
│   ├── outputs.tf               # 출력값
│   └── terraform.tfvars         # 변수값
│
└── docs/                       # 문서
```

---

## 🚀 빠른 시작

### 사전 요구사항

```bash
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# mise (SDK 관리)
mise use -g nodejs

# pnpm
npm install -g pnpm

# worker-build
cargo install worker-build
```

### 환경변수 설정

```bash
# .env 복사 및 설정
cp .env.example .env
```

**필수 환경변수** (.env 파일):

| 변수명 | 설명 | 발급처 |
|--------|------|--------|
| `CLOUDFLARE_API_TOKEN` | Cloudflare API 토큰 | [Cloudflare Dashboard](https://dash.cloudflare.com/profile/api-tokens) |
| `CLOUDFLARE_ACCOUNT_ID` | Cloudflare 계정 ID | Cloudflare Dashboard 사이드바 |
| `PUSHOVER_USER_KEY` | PushOver 사용자 키 | [PushOver](https://pushover.net) |
| `PUSHOVER_API_TOKEN` | PushOver API 토큰 | PushOver Settings → Applications |
| `WEBHOOK_SECRET` | 웹훅 서명용 시크릿 | `openssl rand -base64 32` |

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

## 📋 기능

### Rust SDK (`crates/sdk/`)

```rust
use pushover_sdk::{PushOverClient, Message, Priority};

let client = PushOverClient::new(user_key, api_token);
let msg = Message::builder()
    .message("Hello World")
    .title("알림")
    .priority(Priority::High)
    .build();

let response = client.send(msg).await?;
```

**기능**:

- ✅ PushOver API 메시지 전송
- ✅ 메시지 상태 조회
- ✅ 웹훅 시그니처 검증 (HMAC-SHA256)
- ✅ Feature flag로 환경 분리 (reqwest/cloudflare-worker)

### Rust CLI (`crates/cli/`)

```bash
# 메시지 전송
pushover send "Hello World" --title "Test"
pushover send "긴급" --device iphone --sound siren

# 이력 조회
pushover history --limit 50

# 설정 관리
pushover config set user_key <KEY>
pushover config set token <TOKEN>
```

### Rust Worker (`crates/worker/`)

**API 엔드포인트**:

| Method | Path | 설명 |
| -------- | ------ | ------ |
| `GET` | `/` | 루트 정보 |
| `GET` | `/health` | 헬스체크 |
| `POST` | `/api/v1/messages` | 메시지 전송 |
| `GET` | `/api/v1/messages` | 메시지 목록 조회 |
| `GET` | `/api/v1/messages/:receipt/status` | 수신 상태 조회 |
| `POST` | `/api/v1/webhooks` | PushOver callback 수신 |
| `POST` | `/api/v1/webhooks/register` | Webhook 등록 |
| `GET` | `/api/v1/webhooks` | Webhook 목록 조회 |
| `DELETE` | `/api/v1/webhooks/:id` | Webhook 삭제 |

**기능**:

- ✅ Bearer 토큰 인증 (D1 `api_tokens` 테이블)
- ✅ CORS 지원
- ✅ 실패 메시지 복구 (D1 `failed_deliveries` + Cron Trigger)
- ✅ 웹훅 시그니처 검증 (Timing-safe)

### cURL로 Worker API 테스트

Worker URL: `https://pushover-worker.cromksy.workers.dev`

```bash
# 환경변수 설정
WORKER_URL="https://pushover-worker.cromksy.workers.dev"

# Worker 인증 토큰 (D1 api_tokens 테이블에 등록된 토큰)
API_TOKEN="<your-worker-api-token>"

# PushOver 사용자 키 (PushOver 계정 설정에서 확인)
PUSHOVER_USER_KEY="<your-pushover-user-key>"
```

```bash
# 헬스체크
curl -s "$WORKER_URL/health"

# 메시지 전송
curl -s -X POST "$WORKER_URL/api/v1/messages" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $API_TOKEN" \
  -d '{
    "user": "$PUSHOVER_USER_KEY",
    "message": "Hello from PushOver Worker!",
    "title": "테스트 알림",
    "priority": 0,
    "sound": "pushover"
  }'

# 응답 예시
# {"status":"success","request":"xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx","receipt":"xxxxxxxxxxxxxxxxxxxxxxxx"}

# 우선순위 긴급 메시지 (priority=2, retry/expire 필수)
curl -s -X POST "$WORKER_URL/api/v1/messages" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $API_TOKEN" \
  -d '{
    "user": "$PUSHOVER_USER_KEY",
    "message": "긴급 알림!",
    "title": "URGENT",
    "priority": 2,
    "retry": 60,
    "expire": 3600,
    "sound": "siren"
  }'

# 메시지 목록 조회 (기본 50개)
curl -s "$WORKER_URL/api/v1/messages" \
  -H "Authorization: Bearer $API_TOKEN"

# 메시지 목록 조회 (개수 제한)
curl -s "$WORKER_URL/api/v1/messages?limit=10" \
  -H "Authorization: Bearer $API_TOKEN"

# 메시지 수신 상태 조회
curl -s "$WORKER_URL/api/v1/messages/<receipt>/status" \
  -H "Authorization: Bearer $API_TOKEN"

# 응답 예시
# {"status":"sent","receipt":"xxx","acknowledged":false,"delivered_at":null,"acknowledged_at":null,"created_at":"2026-03-28T12:00:00Z"}

# Webhook 등록
curl -s -X POST "$WORKER_URL/api/v1/webhooks/register" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $API_TOKEN" \
  -d '{
    "url": "https://example.com/webhook",
    "events": "delivered,acknowledged,expired"
  }'

# Webhook 목록 조회
curl -s "$WORKER_URL/api/v1/webhooks" \
  -H "Authorization: Bearer $API_TOKEN"

# Webhook 삭제
curl -s -X DELETE "$WORKER_URL/api/v1/webhooks/<webhook-id>" \
  -H "Authorization: Bearer $API_TOKEN"
```

### Dashboard E2E 테스트 (Playwright)

```bash
cd dashboard

# Playwright 브라우저 설치 (최초 1회)
npx playwright install chromium

# Dashboard dev 서버 실행 (별도 터미널)
pnpm dev

# E2E 테스트 실행
npx playwright test

# UI 모드로 실행 (디버깅)
npx playwright test --ui

# 특정 테스트만 실행
npx playwright test -g "메시지 전송"

# 브라우저 화면 보면서 실행
npx playwright test --headed
```

**테스트 케이스** (`tests/e2e/basic.spec.ts`):

| 테스트 | 설명 |
|--------|------|
| 메인 페이지 로드 | h1 타이틀, "메시지 보내기" 버튼 표시 확인 |
| 메시지 전송 모달 열기 | 모달 오픈 → 제목/메시지 입력 필드 표시 확인 |
| 메시지 전송 | 메시지 입력 → 전송 클릭 → 로딩 완료 확인 |
| History 페이지 이동 | 네비게이션 → `/history` URL + 타이틀 확인 |
| API 키 설정 | `/settings` → API Key 입력 → 저장 클릭 |

```bash
# 실행 결과 예시
$ npx playwright test

Running 5 tests using 1 worker

  ✓ basic.spec.ts:4:3 › PushOver Dashboard › 메인 페이지 로드 (1.2s)
  ✓ basic.spec.ts:11:3 › PushOver Dashboard › 메시지 전송 모달 열기 (0.8s)
  ✓ basic.spec.ts:21:3 › PushOver Dashboard › 메시지 전송 (2.1s)
  ✓ basic.spec.ts:32:3 › PushOver Dashboard › History 페이지 이동 (1.5s)
  ✓ basic.spec.ts:42:3 › Settings › API 키 설정 (1.0s)

  5 passed (6.6s)
```

### Next.js Dashboard (`dashboard/`)

**페이지**:

- `/` - 메인 (메시지 전송 모달 + 통계)
- `/history` - 전송 이력 테이블
- `/settings` - API 키 설정

**기술 스택**:

- Next.js 16 (App Router)
- React 19
- Tailwind CSS 4

---

## 🗄️ 데이터베이스 스키마

### D1 Database

```sql
-- 인증 토큰
CREATE TABLE api_tokens (
    token TEXT PRIMARY KEY,
    user_key TEXT NOT NULL,
    name TEXT,
    active INTEGER DEFAULT 1,
    last_used_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 메시지
CREATE TABLE messages (
    id TEXT PRIMARY KEY,
    user_key TEXT NOT NULL,
    message TEXT NOT NULL,
    title TEXT,
    priority INTEGER DEFAULT 0,
    sound TEXT DEFAULT 'pushover',
    device TEXT,
    url TEXT,
    url_title TEXT,
    html INTEGER DEFAULT 0,
    status TEXT DEFAULT 'pending',   -- pending, sent, failed, delivered, acknowledged
    receipt TEXT,
    api_token TEXT,
    sent_at TEXT,
    delivered_at TEXT,
    acknowledged_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 웹훅
CREATE TABLE webhooks (
    id TEXT PRIMARY KEY,
    user_key TEXT NOT NULL,
    url TEXT NOT NULL,
    secret TEXT NOT NULL,
    events TEXT NOT NULL,   -- "delivered,acknowledged,expired"
    active INTEGER DEFAULT 1,
    last_triggered_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 웹훅 전달 기록
CREATE TABLE webhook_deliveries (
    id TEXT PRIMARY KEY,
    webhook_id TEXT NOT NULL,
    message_id TEXT NOT NULL,
    event_type TEXT NOT NULL,      -- delivered, acknowledged, expired
    status TEXT DEFAULT 'pending', -- pending, delivered, failed
    status_code INTEGER,
    response_body TEXT,
    last_retry_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 실패 메시지 (재시도용)
CREATE TABLE failed_deliveries (
    id TEXT PRIMARY KEY,
    message_id TEXT NOT NULL,
    attempt_count INTEGER DEFAULT 0,
    last_attempt_at TEXT,
    error_message TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

---

## 🔒 보안

**보안 기능**:

- **Timing-safe comparison**: 서명 검증에 상수시간 비교 사용 (Timing Attack 방지)
- **HMAC-SHA256**: 웹훅 서명 검증
- **CORS**: Cross-Origin 요청 제어
- **Bearer Auth**: D1 기반 API 토큰 인증

---

## 📊 개발 현황

| 컴포넌트 | 상태 | 비고 |
| ---------- | ------ | ------ |
| Rust SDK | ✅ 완료 | HTTP 클라이언트, 웹훅 검증 |
| Rust CLI | ✅ 완료 | send, history 명령어 |
| Rust Worker | ✅ 완료 | API 서버, 복구 Cron |
| Dashboard | ✅ 완료 | Next.js 16 UI |
| Infrastructure | ✅ 완료 | OpenTofu (D1, KV, R2, Cron Trigger) |

---

## 🚀 배포

### Infrastructure 배포 (OpenTofu)

```bash
cd infrastructure

# 초기 설정 (최초 1회)
mv backend.tf backend.tf.bak
tofu init -backend=false
tofu apply -target=cloudflare_r2_bucket.terraform_state
mv backend.tf.bak backend.tf
tofu init -reconfigure   # 'yes' to copy state

# 이후 배포
tofu init
tofu plan
tofu apply
```

**관리 리소스**: D1 Database, KV Namespace, R2 Bucket (state), Cron Trigger

### Worker 배포

```bash
cd crates/worker

# 마이그레이션 실행
wrangler d1 execute pushover-db --file=./migrations/0001_init.sql
wrangler d1 execute pushover-db --file=./migrations/0002_add_api_token.sql
wrangler d1 execute pushover-db --file=./migrations/0003_api_tokens.sql

# API 토큰 등록
wrangler d1 execute pushover-db --command="INSERT INTO api_tokens (token, user_key, name) VALUES ('<your-token>', '<your-pushover-user-key>', 'dashboard');"

# 배포
wrangler deploy
```

**배포 URL**: `https://pushover-worker.cromksy.workers.dev`

### Dashboard 배포 (Cloudflare Pages)

```bash
cd dashboard
pnpm run pages:build
pnpm run deploy
```

---

## 📝 라이선스

MIT
