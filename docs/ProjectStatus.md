# PushOver Serverless 플랫폼폄재 작업 현황

최종 수정일: 2026-03-26

## 개요

PushOver 알림 서비스의 Serverless 구현 버전입니다.

---

## 구현 완료 상태

### ✅ 완료된| 컴포넌트 | 상태 | 설명 |
|------|------|------|
| **Rust SDK** | `crates/sdk/` | ✅ 완료 | PushOver API 클라이언트,| **Rust CLI** | `crates/cli/` | ✅ 완료 | 터미널 알림 전송/이력 관리 도구 |
| **Rust Worker** | `crates/worker/` | ✅ 완료 | Cloudflare Worker API 서버 |
| **Next.js Dashboard** | `dashboard/` | ✅ 완료 | 웹 UI (메시지 전송, 이력) |
| **Infrastructure** | `infrastructure/` | ⚠️ 미완 | OpenTofu 구성 (구조만 존재) |
| **E2E Tests** | `dashboard/tests/e2e/` | ⚠️ 미완 | Playwright 테스트 (구조만 존재) |

---

## 배포 상태

### Worker
- **URL**: https://pushover-worker.cromksy.workers.dev
- **상태**: ✅ 정상 배포됨
- **엔드포인트**:
  - `GET /` - 루트 정보
  - `GET /health` - 헬스체크
  - `POST /api/v1/messages` - 메시지 전송
  - `GET /api/v1/messages/:receipt/status` - 상태 조회
  - `POST /api/v1/webhooks` - 웹훅 수신
  - `POST /api/v1/webhooks/register` - 웹훅 등록
  - `GET /api/v1/webhooks` - 웹훅 목록
  - `DELETE /api/v1/webhooks/:id` - 웹훅 삭제

### Dashboard
- **개발**: `http://localhost:3000`
- **배포**: 미배포 (개발 완료 상태)

---

## 기술 스택

| 영역 | 기술 | 버전 |
|------|------|-------|
| SDK | Rust | 1.x |
| CLI | Rust (clap, tokio) | Latest |
| Worker | Rust (worker-rs 0.7) | 0.7 |
| Dashboard | Next.js | 16.x |
| Database | Cloudflare D1 | - |
| Build | worker-build | Latest |

---

## 프로젝트 구조

```
pushover/
├── crates/
│   ├── sdk/                    # PushOver API 클라이언트
│   │   ├── src/
│   │   │   ├── lib.rs           # 공개 API
│   │   │   ├── models.rs        # 데이터 모델
│   │   │   ├── error.rs         # 에러 타입
│   │   │   ├── http_client.rs  # HTTP 클라이언트
│   │   │   └── webhook.rs       # 웹훅 검증
│   │   └── tests/
│   │
│   ├── cli/                    # 터미널 CLI 도구
│   │   ├── src/
│   │   │   ├── main.rs          # 진입점
│   │   │   ├── commands/
│   │   │   │   ├── send.rs      # 메시지 전송
│   │   │   │   └── history.rs    # 이력 조회
│   │   │   └── config.rs        # 설정 관리
│   │
│   └── worker/                 # Cloudflare Worker
│       ├── src/
│       │   ├── lib.rs           # 진입점
│       │   ├── routes/           # API 라우트
│       │   ├── middleware/      # CORS, 인증
│       │   ├── types/           # 요청/응답 타입
│       │   ├── recovery/         # 실패 메시지 복구
│       │   └── utils/           # 유틸리티
│       └── wrangler.toml
│
├── dashboard/                  # Next.js 웹 UI
│   ├── src/
│   │   ├── app/
│   │   │   ├── page.tsx         # 메인 페이지
│   │   │   ├── history/         # 이력 페이지
│   │   │   └── settings/        # 설정 페이지
│   │   └── lib/
│   │       └── api.ts           # API 클라이언트
│   └── package.json
│
├── infrastructure/              # OpenTofu (미완성)
│   └── *.tf
│
├── migrations/                # D1 마이그레이션
│   └── 001_init.sql
│
└── docs/                      # 문서
    └── superpowers/
        └── specs/
            └── 2026-03-26-pushover-serverless-design.md
```

---

## 주요 기능

### 1. Rust SDK (`crates/sdk/`)

**기능**:
- PushOver API 메시지 전송
- 메시지 상태 조회
- 웹훅 시그니처 검증 (HMAC-SHA256)
- Feature flag로 환경 분리 (reqwest/cloudflare-worker)

**주요 타입**:
```rust
pub struct Message {
    pub message: String,
    pub title: Option<String>,
    pub priority: Option<Priority>,
    pub device: Option<String>,
    pub sound: Option<Sound>,
    // ... 기타 옵션들
}
```

### 2. Rust CLI (`crates/cli/`)

**명령어**:
```bash
pushover send "메시지" -t "제목" -p 2
pushover history --limit 50
pushover config set user_key <KEY>
```

**기능**:
- 메시지 전송 (모든 옵션 지원)
- 전송 이력 조회
- 설정 파일 관리 (`~/.config/pushover/config.toml`)

### 3. Rust Worker (`crates/worker/`)

**API 엔드포인트**:
```
POST   /api/v1/messages           # 메시지 전송
GET    /api/v1/messages/:id/status  # 상태 조회
POST   /api/v1/webhooks           # 웹훅 수신
POST   /api/v1/webhooks/register   # 웹훅 등록
GET    /api/v1/webhooks           # 웹훅 목록
DELETE /api/v1/webhooks/:id       # 웹훅 삭제
GET    /health                     # 헬스체크
```

**기능**:
- Bearer 토큰 인증
- CORS 지원
- 메시지 큐잉 처리 (Cloudflare Queues)
- 실패 메시지 복구 (KV 백업 + Cron)
- 웹훅 시그니처 검증

### 4. Next.js Dashboard (`dashboard/`)

**페이지**:
- `/` - 메인 (메시지 전송 모달 + 통계)
- `/history` - 전송 이력 테이블
- `/settings` - API 키 설정

**기능**:
- 메시지 전송 UI
- 전송 이력 조회
- API 키 설정
- TanStack Query로 데이터 캐싱

---

## D1 데이터베이스 스키마

```sql
-- 메시지 테이블
CREATE TABLE messages (
    id TEXT PRIMARY KEY,
    status TEXT NOT NULL DEFAULT 'pending',
    title TEXT,
    message TEXT NOT NULL,
    priority INTEGER DEFAULT 0,
    sound TEXT DEFAULT 'pushover',
    device TEXT,
    pushover_receipt TEXT,
    error TEXT,
    retry_count INTEGER DEFAULT 0,
    created_at INTEGER NOT NULL,
    sent_at INTEGER,
    date TEXT NOT NULL
);

-- 웹훅 테이블
CREATE TABLE webhooks (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    url TEXT NOT NULL,
    secret TEXT NOT NULL,
    active INTEGER DEFAULT 1,
    created_at INTEGER NOT NULL
);
```

---

## 재시도 메커니즘

```
┌──────────────┐
│  메시지 전송  │
└──────┬───────┘
        │
        ▼ (실패)
┌──────────────────┐
│  KV 백업 저장  │
│  (failed:* key) │
└──────┬───────┘
        │
        ▼ (5분마다)
┌──────────────────┐
│ Recovery Worker │
│  (Cron)           │
└──────┬───────┘
        │
        ▼ (재큐잉)
┌──────────────────┐
│  Cloudflare Queue │
└──────────────────┘
```

---

## 다음 단계 (TODO)

### 우선순위 높음
- [ ] Infrastructure OpenTofu 구성 완성
- [ ] E2E 테스트 구현 (Playwright)
- [ ] Dashboard Cloudflare Pages 배포

### 우선순위 중간
- [ ] SDK 첨부파일 업로드 지원
- [ ] CLI 추가 명령어 구현 (webhook, status)
- [ ] Dashboard 인증 (Cloudflare Access)

### 우선순위 낮음
- [ ] Analytics Engine 통합
- [ ] 모니터링/알림 설정
- [ ] API 문서 (OpenAPI Spec)

---

## 최근 커밋 내역

```
7a005e4 docs: 환경변수 설정 문서 추가
b33a956 feat: E2E Tests 구현
0c8f254 feat: Dashboard 구현
8c87059 feat: CLI History Command 구현
8d1970a docs: README & CONTRIBUTING 작성
442a79b feat: OpenTofu Infrastructure 구성
c613b56 feat: D1 Complete Schema 정의
e0ea449 feat: Recovery Worker 구현
09606df feat: Worker webhook CRUD API 구현
```

---

## 관련 문서

- [설계 문서](docs/superpowers/specs/2026-03-26-pushover-serverless-design.md)
- [README.md](../README.md)
- [CONTRIBUTING.md](../CONTRIBUTING.md)
