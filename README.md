# PushOver Serverless Platform

PushOver API를 위한 Rust 기반 Serverless 플랫폼

## 📦 구조

Monorepo 구조로 관리되는 3개의 crate:

- **`crates/sdk`**: Rust SDK (타입 정의, HTTP 클라이언트, 웹훅 검증)
- **`crates/cli`**: CLI 도구 (메시지 전송, 설정 관리)
- **`crates/worker`**: Cloudflare Worker (API 서버, 웹훅 처리)

## 🚀 빠른 시작

### 사전 요구사항

```bash
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Node.js (Worker 빌드용)
mise use -g nodejs

# pnpm
npm install -g pnpm
```

### 환경변수 설정

```bash
# .env 복사 및 설정
cp .env.example .env
# .env 파일에 필요한 토큰 값 입력 (아래 참고)
```

**필수 토큰**:
- `CLOUDFLARE_API_TOKEN`: Cloudflare API 토큰 (Workers, D1, KV 권한)
- `CLOUDFLARE_ACCOUNT_ID`: Cloudflare 계정 ID
- `PUSHOVER_USER_KEY`: PushOver 사용자 키
- `PUSHOVER_API_TOKEN`: PushOver API 토큰

**토큰 발급**:
- Cloudflare: https://dash.cloudflare.com/profile/api-tokens
- PushOver: https://pushover.net (Settings → Applications)

### 설치

```bash
# 의존성 설치
pnpm install

# Worker 빌드
cd crates/worker && pnpm install
pnpm build
```

### CLI 사용

```bash
# 설치
cargo install --path crates/cli

# 설정 초기화
pushover config init

# 메시지 전송
pushover send "Hello, World!" --title "Test"

# 기록 확인
pushover history
```

### Worker 배포

```bash
# D1 데이터베이스 생성
wrangler d1 create pushover-db

# 마이그레이션 실행
wrangler d1 execute pushover-db --file=./migrations/0001_init.sql

# Worker 배포
cd crates/worker
wrangler deploy
```

## 🏗️ 아키텍처

```
┌─────────────┐      ┌──────────────┐      ┌─────────────┐
│   CLI       │──────│   Worker     │──────│  PushOver   │
│  (Client)   │      │  (Cloudflare)│      │    API      │
└─────────────┘      └──────────────┘      └─────────────┘
                            │
                            ▼
                     ┌──────────────┐
                     │  D1 Database │
                     │  (Messages,  │
                     │   Webhooks)  │
                     └──────────────┘
```

## 📋 기능

- ✅ 메시지 전송 (PushOver API 통합)
- ✅ 웹훅 수신 및 검증 (HMAC-SHA256)
- ✅ 메시지 상태 추적
- ✅ 실패한 메시지 자동 재시도
- ⏳ Dashboard (개발 중)

## 🔒 보안

- **Timing-safe comparison**: 서명 검증에 상수시간 비교 사용
- **HMAC-SHA256**: 웹훅 서명 검증
- **CORS**: Cross-Origin 요청 제어
- **인증**: API 키 기반 인증

## 📝 라이선스

MIT
