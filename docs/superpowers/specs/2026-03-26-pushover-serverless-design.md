# PushOver Serverless 플랫폼 설계 문서

## 개요

Cloudflare Serverless 스택을 활용한 PushOver 알림 플랫폼입니다. Rust SDK, CLI 도구, 그리고 웹 대시보드를 포함한 통합 솔루션입니다.

### 기술 스택

| 영역 | 기술 |
|------|------|
| SDK/CLI | Rust (WebAssembly) |
| Backend | Cloudflare Workers (TypeScript) |
| Frontend | Cloudflare Pages (React/TypeScript) |
| Database | Cloudflare D1 |
| Cache/Queue | Cloudflare KV, Queues |
| IaC | OpenTofu |
| Auth | Cloudflare Access |

---

## 아키텍처

### 프로젝트 구조

```
pushover/
├── crates/                          # Rust 워크스페이스
│   ├── sdk/                         # PushOver SDK (코어 라이브러리)
│   │   ├── src/
│   │   │   ├── api.rs               # PushOver API 클라이언트
│   │   │   ├── webhook.rs           # 웹훅 수신/처리
│   │   │   ├── cloudflare/          # Cloudflare 바인딩 모듈
│   │   │   │   ├── kv.rs
│   │   │   │   ├── d1.rs
│   │   │   │   ├── queues.rs
│   │   │   │   └── hyperdrive.rs
│   │   │   └── lib.rs
│   │   └── Cargo.toml
│   ├── cli/                         # CLI 도구
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── commands/            # send, history, config, webhook
│   │   │   └── config/
│   │   └── Cargo.toml
│   └── Cargo.toml                   # 워크스페이스 루트
│
├── cloudflare/                       # TypeScript 서버리스
│   ├── worker/                      # Cloudflare Worker
│   │   ├── src/
│   │   │   ├── handlers/            # API, 웹훅 핸들러
│   │   │   ├── queues/              # 큐 컨슈머
│   │   │   ├── recovery/            # Recovery Worker
│   │   │   └── index.ts
│   │   ├── wrangler.toml
│   │   └── package.json
│   │
│   ├── dashboard/                   # Cloudflare Pages (React/TypeScript)
│   │   ├── src/
│   │   │   ├── components/          # 대시보드 UI
│   │   │   ├── pages/               # 전송(모달), 히스토리, 설정
│   │   │   ├── api/                 # Backend API 호출
│   │   │   └── main.tsx
│   │   └── package.json
│   │
│   └── openapi/                     # API 문서 (OpenAPI Spec)
│       └── spec.yaml                # Swagger UI용 스펙
│
├── infra/                           # OpenTofu (IaC)
│   ├── main.tf                      # 메인 구성
│   ├── variables.tf                 # 변수 정의
│   ├── outputs.tf                   # 출력값
│   ├── modules/
│   │   └── cloudflare/
│   │       ├── worker.tf
│   │       ├── pages.tf
│   │       ├── d1.tf
│   │       ├── kv.tf
│   │       └── queues.tf
│   └── secrets/
│       └── auto.tfvars.example      # 시크릿 템플릿
│
└── docs/                            # 문서
    ├── README.md
    └── api/                         # API 문서 (자동 생성)
```

### 데이터 흐름

```
┌─────────────────────────────────────────────────────────────────┐
│                        CLI / Dashboard                          │
└───────────────────────────┬─────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Cloudflare Worker                            │
│  ┌─────────────┐    ┌──────────────┐    ┌──────────────────┐   │
│  │   API       │    │   Webhook    │    │   Recovery       │   │
│  │  Handler    │───▶│  Handler     │───▶│   Worker         │   │
│  └──────┬──────┘    └──────┬───────┘    └────────┬─────────┘   │
│         │                  │                      │             │
│         ▼                  ▼                      ▼             │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                    Cloudflare Queues                     │  │
│  │   ┌─────────┐    ┌─────────┐    ┌──────────────────┐   │  │
│  │   │ Send    │    │ Webhook │    │   Retry          │   │  │
│  │   │ Queue   │    │ Queue   │    │   Queue          │   │  │
│  │   └────┬────┘    └────┬────┘    └────────┬─────────┘   │  │
│  └────────┼──────────────┼──────────────────┼──────────────┘  │
│           │              │                  │                │
│           ▼              ▼                  ▼                │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                        D1                                │  │
│  │   ┌────────────┐  ┌────────────┐  ┌──────────────────┐ │  │
│  │   │ messages   │  │ history    │  │ webhooks         │ │  │
│  │   └────────────┘  └────────────┘  └──────────────────┘ │  │
│  └──────────────────────────────────────────────────────────┘  │
│                              │                                 │
│                              ▼                                 │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                    Cloudflare KV                         │  │
│  │   ┌──────────────────────────────────────────────────┐  │  │
│  │   │ failed:* (백업)                                   │  │  │
│  │   └──────────────────────────────────────────────────┘  │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                      PushOver API                               │
└─────────────────────────────────────────────────────────────────┘
```

---

## 컴포넌트 상세

### 1. Rust SDK (crates/sdk)

PushOver API 클라이언트 및 Cloudflare 바인딩을 제공하는 풀 스택 SDK입니다.

#### 모듈 구조

```rust
pub mod api;           // PushOver API 클라이언트
pub mod webhook;       // 웹훅 처리
pub mod cloudflare;    // Cloudflare 바인딩
pub mod error;         // 에러 타입
pub mod types;         // 공통 타입
```

#### 핵심 API

```rust
pub struct PushOverClient {
    user_key: String,
    token: String,
    http: HttpClient,
}

impl PushOverClient {
    pub async fn send(&self, msg: Message) -> Result<SendResponse>;
    pub async fn send_with_attachment(&self, msg: Message, file: File) -> Result<SendResponse>;
    pub async fn validate_user(&self) -> Result<bool>;
}

pub struct Message {
    pub message: String,
    pub title: Option<String>,
    pub priority: Option<Priority>,    // -2 ~ 2
    pub sound: Option<Sound>,
    pub device: Option<String>,
    pub expire: Option<u32>,           // TTL for emergency
    pub retry: Option<u32>,            // Retry interval
}
```

#### 아키텍처 (Feature Flag 분리)

SDK는 **두 가지 실행 환경**을 지원합니다:

| Feature | 환경 | 구현 방식 |
|---------|------|----------|
| `cloudflare-worker` | Cloudflare Worker 내부 | `wasm-bindgen`으로 JS 호스트 API 직접 호출 |
| (default) | CLI / 외부 | `reqwest` HTTP 클라이언트 |

```rust
// crates/sdk/src/lib.rs
#[cfg(feature = "cloudflare-worker")]
pub mod cloudflare_bindings;
#[cfg(not(feature = "cloudflare-worker"))]
pub mod http_client;

// 공통 트레이트
pub trait Storage {
    async fn get(&self, key: &str) -> Result<Option<String>>;
    async fn put(&self, key: &str, value: &str, ttl: Option<u32>) -> Result<()>;
    async fn delete(&self, key: &str) -> Result<()>;
}

// Worker 환경용 구현
#[cfg(feature = "cloudflare-worker")]
pub struct WorkerStorage { binding: String }

#[cfg(feature = "cloudflare-worker")]
impl Storage for WorkerStorage {
    #[wasm_bindgen]
    pub async fn get(&self, key: &str) -> Result<Option<String>> {
        // js_sys로 KV.get 호출
    }
}

// CLI 환경용 구현 (HTTP API 통해 Worker 호출)
#[cfg(not(feature = "cloudflare-worker"))]
pub struct HttpStorage { base_url: String, api_key: String }

#[cfg(not(feature = "cloudflare-worker"))]
impl Storage for HttpStorage {
    pub async fn get(&self, key: &str) -> Result<Option<String>> {
        // reqwest로 Worker API 호출
    }
}
```

#### 의존성

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"

# CLI/외부 환경용
reqwest = { version = "0.11", features = ["json", "multipart"], optional = true }

# Cloudflare Worker 환경용
wasm-bindgen = { version = "0.2", optional = true }
wasm-bindgen-futures = { version = "0.4", optional = true }
js-sys = { version = "0.3", optional = true }

[features]
default = ["reqwest"]
cloudflare-worker = ["wasm-bindgen", "wasm-bindgen-futures", "js-sys"]
```

---

### 2. CLI 도구 (crates/cli)

터미널에서 PushOver 알림을 관리하는 명령행 도구입니다.

#### 명령어 구조

```bash
pushover [COMMAND] [OPTIONS]

Commands:
  send        메시지 전송
  history     전송 이력 조회
  config      설정 관리
  webhook     웹훅 관리
  dashboard   대시보드 URL 열기
  status      계정 상태 확인
```

#### 상세 명령어

```bash
# 메시지 전송
pushover send "Hello World"
pushover send -t "제목" -p 2 -s pushover "긴급 메시지"
pushover send --device iphone "특정 기기만"
pushover send -f image.png "이미지와 함께"

# 전송 이력
pushover history              # 최근 20개
pushover history --limit 50   # 50개
pushover history --today      # 오늘만
pushover history --failed     # 실패만

# 설정 관리
pushover config set user_key <KEY>
pushover config set token <TOKEN>
pushover config set api_url <URL>
pushover config list
pushover config init           # 대화형 설정

# 웹훅 관리
pushover webhook list
pushover webhook create --name "github" --url "https://..."
pushover webhook delete <ID>
pushover webhook test <ID>

# 유틸리티
pushover dashboard             # 브라우저로 대시보드 열기
pushover status                # API 연결 확인
```

#### 설정 파일

```toml
# ~/.config/pushover/config.toml
[default]
user_key = "${PUSHOVER_USER_KEY}"  # 환경 변수 참조
token = "${PUSHOVER_TOKEN}"
api_url = "https://api.pushover.net/1"

[profiles.work]
user_key = "${WORK_PUSHOVER_USER_KEY}"
token = "${WORK_PUSHOVER_TOKEN}"

[profiles.personal]
user_key = "${PERSONAL_PUSHOVER_USER_KEY}"
token = "${PERSONAL_PUSHOVER_TOKEN}"
```

---

### 3. Cloudflare Worker (cloudflare/worker)

API 서버 및 웹훅 수신 처리를 담당합니다.

#### API 엔드포인트

```
POST   /api/v1/send              # 메시지 전송
GET    /api/v1/messages          # 메시지 목록
GET    /api/v1/messages/:id      # 메시지 상세
POST   /webhook/:id              # 웹훅 수신
GET    /api/v1/webhooks          # 웹훅 목록
POST   /api/v1/webhooks          # 웹훅 생성
DELETE /api/v1/webhooks/:id      # 웹훅 삭제
GET    /health                   # 헬스체크
GET    /openapi.json             # OpenAPI 스펙
```

#### 입력 검증 (Zod 스키마)

```typescript
import { z } from 'zod';

// 메시지 전송 요청 검증
const SendMessageSchema = z.object({
  message: z.string().min(1).max(1024),
  title: z.string().max(250).optional(),
  priority: z.number().int().min(-2).max(2).optional().default(0),
  sound: z.enum([
    'pushover', 'bike', 'bugle', 'cashregister', 'classical', 'cosmic',
    'falling', 'gamelan', 'incoming', 'intermission', 'magic', 'mechanical',
    'pianobar', 'siren', 'spacealarm', 'tugboat', 'alien', 'climb',
    'persistent', 'echo', 'updown', 'vibrate', 'none'
  ]).optional().default('pushover'),
  device: z.string().max(25).optional(),
  expire: z.number().int().min(30).max(10800).optional(),
  retry: z.number().int().min(30).max(10800).optional(),
  timestamp: z.number().int().positive().optional(),
  url: z.string().url().optional(),
  url_title: z.string().max(100).optional(),
  html: z.string().max(10000).optional(),
  monospace: z.boolean().optional(),
  image_url: z.string().url().optional(),
  attachment: z.string().optional(),
});

// API 에러 응답 포맷
interface ApiErrorResponse {
  error: {
    code: string;      // VALIDATION_ERROR, RATE_LIMITED, UNAUTHORIZED, INTERNAL_ERROR
    message: string;
    details?: Record<string, unknown>;
  };
}
```

#### Worker 구조 (인증 + Rate Limiting)

```typescript
import { Hono } from 'hono';
import { cors } from 'hono/cors';
import { logger } from 'hono/logger';
import { prettyJSON } from 'hono/pretty-json';
import { bearerAuth } from 'hono/bearer-auth';

type Bindings = {
  PUSHOVER_USER_KEY: string;
  PUSHOVER_TOKEN: string;
  API_KEY: string;              // 대시보드 API 인증용
  WEBHOOK_SECRET: string;       // 웹훅 시그니처 검증용
  DB: D1Database;
  QUEUE: Queue;
  KV: KVNamespace;
};

const app = new Hono<{ Bindings: Bindings }>();
app.use('*', logger());
app.use('*', prettyJSON());

// CORS: 프로덕션에서는 도메인 제한 권장
app.use('*', cors({
  origin: (origin) => {
    const allowedOrigins = [
      'https://dashboard.pushover.example.com',
      'http://localhost:5173',
    ];
    return allowedOrigins.includes(origin) ? origin : allowedOrigins[0];
  }
}));

// API 키 인증 미들웨어
const apiKeyAuth = bearerAuth({ verifyToken: async (token, c) => {
  return token === c.env.API_KEY;
}});

// Rate Limiting: Cloudflare Ruleset을 기본 방어로 사용
// KV 기반은 보조 수단 (Race Condition 가능성 있으므로 과감시 용도)
const rateLimiter = async (c: Context, next: Next) => {
  const ip = c.req.header('CF-Connecting-IP') || 'unknown';
  const key = `ratelimit:${ip}`;
  const limit = 100;  // 분당 100회
  const window = 60;  // 60초 윈도우

  // 주의: KV get-put 사이 Race Condition 가능성
  // Cloudflare Ruleset이 1차 방어, 이것은 모니터링/과감시용
  const current = await c.env.KV.get(key, 'json');
  const now = Math.floor(Date.now() / 1000);

  let count = 0;
  if (current) {
    if (current.windowStart + window > now) {
      count = current.count + 1;
    } else {
      count = 1;
    }
  }

  if (count > limit) {
    // 로깅 (모니터링용)
    console.warn(JSON.stringify({
      level: 'warn',
      message: 'Rate limit exceeded',
      ip,
      count,
      limit,
      timestamp: now
    }));

    return c.json<ApiErrorResponse>({
      error: {
        code: 'RATE_LIMITED',
        message: 'Too many requests. Please try again later.',
        details: { limit, window, current: count }
      }
    }, 429);
  }

  await c.env.KV.put(key, JSON.stringify({ count, windowStart: now }), {
    expirationTtl: window
  });

  await next();
};

// 메시지 전송 (인증 + Rate Limiting 적용)
app.post('/api/v1/send', apiKeyAuth, rateLimiter, async (c) => {
  const body = await c.req.json();

  // Zod 검증
  const result = SendMessageSchema.safeParse(body);
  if (!result.success) {
    return c.json<ApiErrorResponse>({
      error: {
        code: 'VALIDATION_ERROR',
        message: 'Invalid request body',
        details: result.error.flatten()
      }
    }, 400);
  }

  const messageId = crypto.randomUUID();

  await c.env.QUEUE.send({
    type: 'send',
    messageId,
    payload: result.data,
    timestamp: Date.now(),
  });

  await c.env.DB.prepare(`
    INSERT INTO messages (id, status, payload, created_at)
    VALUES (?, 'pending', ?, ?)
  `).bind(messageId, JSON.stringify(result.data), Date.now()).run();

  return c.json({ success: true, messageId });
});

// 웹훅 수신 (시그니처 검증)
app.post('/webhook/:id', async (c) => {
  const webhookId = c.req.param('id');
  const signature = c.req.header('X-Webhook-Signature');
  const body = await c.req.text();

  // HMAC-SHA256 시그니처 검증
  const encoder = new TextEncoder();
  const key = await crypto.subtle.importKey(
    'raw',
    encoder.encode(c.env.WEBHOOK_SECRET),
    { name: 'HMAC', hash: 'SHA-256' },
    false,
    ['sign']
  );
  const sigBuffer = await crypto.subtle.sign('HMAC', key, encoder.encode(body));
  const hexSignature = Array.from(new Uint8Array(sigBuffer))
    .map(b => b.toString(16).padStart(2, '0'))
    .join('');

  // Timing-safe 비교 (Timing Attack 방지)
  // 일반 문자열 비교는 실패 시 즉시 반환하여 타이밍 차이 발생
  const sigBytes = hexToBytes(signature);
  const expectedBytes = hexToBytes(hexSignature);

  if (sigBytes.length !== expectedBytes.length || !timingSafeEqual(sigBytes, expectedBytes)) {
    return c.json<ApiErrorResponse>({
      error: {
        code: 'UNAUTHORIZED',
        message: 'Invalid webhook signature'
      }
    }, 401);
  }

// Timing-safe 비교 함수
function hexToBytes(hex: string): Uint8Array {
  const bytes = new Uint8Array(hex.length / 2);
  for (let i = 0; i < hex.length; i += 2) {
    bytes[i / 2] = parseInt(hex.substr(i, 2), 16);
  }
  return bytes;
}

function timingSafeEqual(a: Uint8Array, b: Uint8Array): boolean {
  let result = 0;
  for (let i = 0; i < a.length; i++) {
    result |= a[i] ^ b[i];
  }
  return result === 0;
}

  await c.env.QUEUE.send({
    type: 'webhook',
    webhookId,
    payload: JSON.parse(body),
    timestamp: Date.now(),
  });

  return c.json({ success: true });
});

export default app;
```

#### Cloudflare Rate Limiting Rules (대안)

```hcl
# infra/modules/cloudflare/rate_limit.tf
resource "cloudflare_ruleset" "rate_limit" {
  account_id  = var.account_id
  name        = "pushover-rate-limit"
  description = "Rate limiting for PushOver API"
  kind        = "zone"
  zone_id     = var.zone_id
  phase       = "http_request"

  rules {
    action = "block"
    action_parameters {
      response {
        status_code = 429
        content_type = "application/json"
        content     = "{\"error\":{\"code\":\"RATE_LIMITED\",\"message\":\"Too many requests\"}}"
      }
    }
    expression = "rate_limit:ip_per_minute > 100"
    description = "Block IPs exceeding 100 req/min"
    enabled = true
  }
}
```

#### Queue Consumer

```typescript
export async function queueConsumer(
  batch: MessageBatch<QueueMessage>,
  env: Bindings
): Promise<void> {
  for (const msg of batch.messages) {
    try {
      switch (msg.body.type) {
        case 'send':
          await processSend(env, msg.body);
          break;
        case 'webhook':
          await processWebhook(env, msg.body);
          break;
        case 'retry':
          await processRetry(env, msg.body);
          break;
      }
      msg.ack();
    } catch (error) {
      msg.retry({ delaySeconds: 60 });
    }
  }
}
```

#### Recovery Worker (Cron)

```typescript
export async function scheduled(event: ScheduledEvent, env: Bindings) {
  const failedKeys = await env.KV.list({ prefix: 'failed:' });

  for (const key of failedKeys.keys) {
    const data = await env.KV.get(key.name, 'json');
    if (!data) continue;

    await env.QUEUE.send({
      type: 'retry',
      originalMessageId: key.name.replace('failed:', ''),
      payload: data.payload,
      retryCount: (data.retryCount || 0) + 1,
    });

    await env.KV.delete(key.name);
  }
}
```

---

### 4. 웹 대시보드 (cloudflare/dashboard)

React/TypeScript 기반의 웹 UI입니다.

#### 페이지 구조

```
/                 # 메인 - 히스토리 + 통계 대시보드
/messages         # 메시지 목록 (상세)
/webhooks         # 웹훅 관리
/settings         # 설정 (API 키, 알림 설정)
/api-docs         # Swagger UI (API 문서)
```

#### 기술 스택

```json
{
  "dependencies": {
    "react": "^18",
    "react-router-dom": "^6",
    "tanstack/react-query": "^5",
    "tailwindcss": "^3",
    "lucide-react": "icons",
    "swagger-ui-react": "API docs"
  }
}
```

#### 주요 기능

- **전송 모달**: 제목, 메시지, 우선순위, 사운드, 기기 선택
- **히스토리 테이블**: 전송 이력 조회, 필터링
- **통계 카드**: 오늘/주간 전송 수, 성공률
- **웹훅 관리**: 생성, 삭제, 테스트
- **API 문서**: Swagger UI로 외부 연동 가이드

---

### 5. D1 데이터베이스 스키마

```sql
-- 메시지 테이블
CREATE TABLE IF NOT EXISTS messages (
    id TEXT PRIMARY KEY,
    status TEXT NOT NULL DEFAULT 'pending',
    title TEXT,
    message TEXT NOT NULL,
    priority INTEGER DEFAULT 0,
    sound TEXT DEFAULT 'pushover',
    device TEXT,
    pushover_request TEXT,
    pushover_receipt TEXT,
    error TEXT,
    retry_count INTEGER DEFAULT 0,
    created_at INTEGER NOT NULL,
    sent_at INTEGER,
    date TEXT NOT NULL
);

CREATE INDEX idx_messages_status ON messages(status);
CREATE INDEX idx_messages_created ON messages(created_at DESC);
CREATE INDEX idx_messages_date ON messages(date);  -- 날짜별 조회
CREATE INDEX idx_messages_status_date ON messages(status, date);  -- 실패 메시지 조회
CREATE INDEX idx_messages_receipt ON messages(pushover_receipt) WHERE pushover_receipt IS NOT NULL;  -- Emergency 확인용

-- 웹훅 테이블
CREATE TABLE IF NOT EXISTS webhooks (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    url TEXT NOT NULL,
    secret TEXT NOT NULL,
    config TEXT,
    active INTEGER DEFAULT 1,
    created_at INTEGER NOT NULL,
    last_triggered_at INTEGER
);

-- 웹훅 이벤트 로그
CREATE TABLE IF NOT EXISTS webhook_events (
    id TEXT PRIMARY KEY,
    webhook_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    payload TEXT,
    status TEXT NOT NULL,
    error TEXT,
    created_at INTEGER NOT NULL,
    processed_at INTEGER,
    FOREIGN KEY (webhook_id) REFERENCES webhooks(id)
);

-- 통계 집계 (일별)
CREATE TABLE IF NOT EXISTS daily_stats (
    date TEXT PRIMARY KEY,
    total_sent INTEGER DEFAULT 0,
    total_failed INTEGER DEFAULT 0,
    webhooks_received INTEGER DEFAULT 0
);

-- 설정 테이블
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at INTEGER NOT NULL
);
```

---

### 6. OpenTofu 인프라

#### 디렉토리 구조

```
infra/
├── main.tf                    # 메인 구성
├── variables.tf               # 변수 정의
├── outputs.tf                 # 출력값
├── providers.tf               # 프로바이더 설정
├── modules/
│   └── cloudflare/
│       ├── worker.tf          # Workers 구성
│       ├── pages.tf           # Pages 구성
│       ├── d1.tf              # D1 데이터베이스
│       ├── kv.tf              # KV 네임스페이스
│       ├── queues.tf          # Queues 구성
│       └── access.tf          # Access 정책
└── environments/
    ├── dev/
    └── prod/
```

#### 주요 리소스

- **Worker Script**: API 서버, 웹훅 핸들러
- **Worker Cron**: Recovery Worker (5분마다)
- **Pages Project**: 대시보드 웹앱
- **D1 Database**: 메시지/웹훅 저장
- **KV Namespace**: 실패 메시지 백업
- **Queues**: 메시지 처리 큐
- **Access Application**: 대시보드 인증

#### 인증 정책

- **Google 로그인**: 허용된 Google Workspace 도메인
- **이메일 OTP**: (선택) 특정 이메일 주소

---

### 7. 재시도 메커니즘

```
┌─────────────────────────────────────────────────────────────┐
│  웹훅 수신                                                   │
└──────────────────┬──────────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────────────┐
│  Cloudflare Queues                                           │
│  ├─ 자동 재시도 (최대 5회)                                    │
│  ├─ Exponential Backoff                                     │
│  └─ 최대 재시도 초과 시 → Dead Letter Queue (DLQ)           │
└──────────────────┬──────────────────────────────────────────┘
                   │
        ┌──────────┴──────────┐
        │                     │
        ▼                     ▼
   [성공]               [모든 재시도 실패]
        │                     │
        │                     ▼
        │         ┌─────────────────────────────────┐
        │         │  KV에 실패 기록 저장             │
        │         │  └─ 재시도 가능한 메타데이터    │
        │         └─────────────────────────────────┘
        │                     │
        │                     ▼
        │         ┌─────────────────────────────────┐
        │         │  Recovery Worker (Cron)         │
        │         │  └─ 주기적으로 KV에서 읽어       │
        │         │     재큐하거나 알림 전송         │
        │         └─────────────────────────────────┘
```

---

### 8. CI/CD 파이프라인

#### GitHub Actions 워크플로우

- **CI**: Rust 테스트, TypeScript 테스트, OpenTofu 검증
- **Deploy**: Worker 배포, Pages 배포 (GitHub 연동), OpenTofu 적용
- **Release**: CLI 바이너리 빌드 (Linux/macOS/Windows)

#### 배포 프로세스

1. `main` 브랜치에 푸시 시 자동 배포
2. Worker: `wrangler deploy`
3. Pages: GitHub 연동으로 자동 빌드/배포
4. OpenTofu: `tofu apply`

---

## 보안 고려사항

1. **API 키 관리**: 환경 변수 또는 Cloudflare Secrets 사용
2. **웹훅 시그니처**: HMAC-SHA256 검증
3. **Access 인증**: Cloudflare Access로 대시보드 보호
4. **HTTPS 강제**: 모든 엔드포인트 HTTPS만 허용
5. **Rate Limiting**: Worker 레벨에서 요청 제한

---

## 테스트 전략

### SDK 테스트 (Rust)

```rust
// crates/sdk/tests/api_test.rs
use wiremock::{MockServer, Mock, ResponseTemplate};
use pushover_sdk::{PushOverClient, Message, Priority};

#[tokio::test]
async fn test_send_message_success() {
    let mock_server = MockServer::start().await;

    Mock::given(wiremock::matchers::method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": 1,
            "request": "test-request-id"
        })))
        .mount(&mock_server)
        .await;

    let client = PushOverClient::new("test_user_key", "test_token");
    let msg = Message::builder()
        .message("Test message")
        .title("Test title")
        .build();

    let result = client.send(msg).await.unwrap();
    assert_eq!(result.status, 1);
}
```

### Worker 테스트 (miniflare)

```typescript
// cloudflare/worker/src/__tests__/api.test.ts
import { env, createExecutionContext, waitOnExecutionContext } from 'cloudflare:test';
import { describe, it, expect } from 'vitest';
import worker from '../index';

describe('API Tests', () => {
  it('POST /api/v1/send - should create message', async () => {
    const ctx = createExecutionContext({
      API_KEY: 'test-api-key',
      DB: env.D1,
      QUEUE: env.QUEUE,
      KV: env.KV,
    });

    const response = await worker.fetch(
      new Request('http://localhost/api/v1/send', {
        method: 'POST',
        headers: { 'Authorization': 'Bearer test-api-key' },
        body: JSON.stringify({ message: 'Hello', title: 'Test' }),
      }),
      ctx
    );

    expect(response.status).toBe(200);
    const body = await response.json();
    expect(body.success).toBe(true);
    expect(body.messageId).toBeDefined();
  });
});
```

### Dashboard 테스트 (React Testing Library)

```typescript
// cloudflare/dashboard/src/__tests__/SendModal.test.tsx
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { SendModal } from '../components/SendModal';

describe('SendModal', () => {
  it('should send message with required fields', async () => {
    const onClose = vi.fn();
    const queryClient = new QueryClient();

    render(
      <QueryClientProvider client={queryClient}>
        <SendModal open onClose={onClose} />
      </QueryClientProvider>
    );

    fireEvent.change(screen.getByLabelText('메시지'), {
      target: { value: 'Test message' },
    });

    fireEvent.click(screen.getByText('전송'));

    await waitFor(() => {
      expect(onClose).toHaveBeenCalled();
    });
  });
});
```

### E2E 테스트 (Playwright)

```typescript
// e2e/send-message.spec.ts
import { test, expect } from '@playwright/test';

test('send notification via dashboard', async ({ page }) => {
  await page.goto('https://dashboard.pushover.example.com');

  // Cloudflare Access 로그인 (필요시)
  // await page.click('text=Sign in with Google');

  // 전송 모달 열기
  await page.click('button:has-text("메시지 전송")');

  // 폼 작성
  await page.fill('input[label="제목"]', 'E2E Test');
  await page.fill('textarea[label="메시지"]', 'Test message from Playwright');
  await page.selectOption('select[label="우선순위"]', '1');

  // 전송
  await page.click('button:has-text("전송")');

  // 성공 확인
  await expect(page.locator('text=전송 완료')).toBeVisible();
});
```

---

## 모니터링/로깅

### Cloudflare Analytics Engine

```typescript
// cloudflare/worker/src/analytics.ts
interface AnalyticsBindings {
  ANALYTICS: AnalyticsEngine;
}

// 이벤트 기록
export async function trackEvent(
  env: AnalyticsBindings,
  event: {
    type: 'message_sent' | 'message_failed' | 'webhook_received' | 'webhook_failed';
    messageId?: string;
    webhookId?: string;
    metadata?: Record<string, string>;
  }
) {
  await env.ANALYTICS.writeDataPoint({
    indexes: [event.type],
    blobs: [
      event.messageId || '',
      event.webhookId || '',
      JSON.stringify(event.metadata || {}),
    ],
    doubles: [Date.now() / 1000],
  });
}

// 쿼리 예시: 시간대별 전송 통계
export async function getMessageStats(env: AnalyticsBindings, hours: number = 24) {
  const result = await env.ANALYTICS.query(`
    SELECT
      blob1 as type,
      COUNT() as count,
      AVG(double1) as avg_timestamp
    FROM analytics
    WHERE double1 > NOW() - INTERVAL '${hours} HOURS'
    GROUP BY blob1
    ORDER BY count DESC
  `);

  return result.data;
}
```

### 로깅 전략

```typescript
// cloudflare/worker/src/logger.ts
type LogLevel = 'debug' | 'info' | 'warn' | 'error';

interface LogEntry {
  timestamp: number;
  level: LogLevel;
  message: string;
  requestId?: string;
  metadata?: Record<string, unknown>;
}

export class Logger {
  constructor(private context: string) {}

  log(level: LogLevel, message: string, metadata?: Record<string, unknown>) {
    const entry: LogEntry = {
      timestamp: Date.now(),
      level,
      message: `[${this.context}] ${message}`,
      metadata,
    };

    // Worker console.log는 Cloudflare 대시보드에서 확인 가능
    console.log(JSON.stringify(entry));

    // 에러 레벨은 Analytics Engine에도 기록
    if (level === 'error') {
      // trackEvent 호출
    }
  }

  debug(msg: string, metadata?: Record<string, unknown>) { this.log('debug', msg, metadata); }
  info(msg: string, metadata?: Record<string, unknown>) { this.log('info', msg, metadata); }
  warn(msg: string, metadata?: Record<string, unknown>) { this.log('warn', msg, metadata); }
  error(msg: string, metadata?: Record<string, unknown>) { this.log('error', msg, metadata); }
}
```

### 알림 설정 (메타 모니터링)

```typescript
// cloudflare/worker/src/alerts.ts
// 에러율 임계치 초과 시 자체 PushOver로 알림
export async function checkErrorRate(env: Bindings) {
  const stats = await getMessageStats(env, 1);  // 지난 1시간

  const total = stats.reduce((sum, s) => sum + s.count, 0);
  const errors = stats.find(s => s.type === 'message_failed')?.count || 0;

  const errorRate = total > 0 ? errors / total : 0;

  if (errorRate > 0.1) {  // 10% 초과
    // 자체 PushOver로 알림
    await fetch('https://api.pushover.net/1/messages.json', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        user: env.PUSHOVER_USER_KEY,
        token: env.PUSHOVER_TOKEN,
        title: '⚠️ PushOver Service Alert',
        message: `Error rate exceeded: ${(errorRate * 100).toFixed(1)}% (last hour)\nTotal: ${total}, Errors: ${errors}`,
        priority: 1,
      }),
    });
  }
}
```

---

## 확장성

- **Hyperdrive**: 외부 PostgreSQL 연결 시 사용 가능
- **R2**: 대용량 첨부 파일 저장
- **Analytics Engine**: 메시지 통계 분석 (Workers Free 플랜: 월 1000만 쿼리 무료)
- **Workers AI**: 메시지 분류, 스팸 필터링

---

## 마일스톤

1. **Phase 1**: Rust SDK + CLI 기본 기능
2. **Phase 2**: Cloudflare Worker API 서버
3. **Phase 3**: 웹 대시보드
4. **Phase 4**: 웹훅 수신 처리
5. **Phase 5**: OpenTofu 인프라 자동화

---

## 참고 자료

- [PushOver API 문서](https://pushover.net/api)
- [Cloudflare Workers 문서](https://developers.cloudflare.com/workers/)
- [OpenTofu 문서](https://opentofu.org/docs/)
