# Roadmap

## v1.x - PushOver wrapper (현재)

### 현재 상태 (v0.1.0)

- ✅ Rust SDK
- ✅ Rust CLI
- ✅ Rust Worker API
- ✅ Next.js Dashboard
- ✅ D1 Database 마이그레이션
- ✅ OpenTofu 인프라
- ✅ Playwright E2E 테스트
- ✅ Webhook (등록/전송/기록)

### v0.2.0 - Queue & KV (Producer-Consumer) ✅

**아키텍처**: Queue-First + KV Fallback

```
메시지 전송:
  Client → POST /messages → Queue 투입 → 202 Accepted
  Consumer → PushOver API 호출
    성공: D1 저장 (status=sent)
    실패: KV 백업 + D1 failed_deliveries

복구:
  Cron (*/5분) → D1 failed_deliveries 조회 → KV에서 본문 복원 → 재전송
  3회 초과 시: KV TTL(7d) 만료로 자동 정리
```

**Queue**:

- [x] 메시지 Producer (POST /messages → Queue 투입)
- [x] 메시지 Consumer (Queue → PushOver API 호출)
- [x] 202 Accepted 즉시 응답

**KV** (3가지 용도):

- [x] Token 캐시: `pushover-tokens:{hash}` → user_key (TTL 1h, D1이 원본)
- [ ] Webhook 캐시: `pushover-webhooks:{user_key}` → webhook 목록 JSON (TTL 5m)
- [x] 실패 메시지 백업: `pushover-failed:{message_id}` → 메시지 본문 (TTL 7d)

**R2** (2가지 용도):

- [x] D1 백업 스냅샷: Cron으로 주기적 D1 export → R2 저장 (재해 복구)
- [x] 메시지 이미지 첨부: 이미지 업로드 → R2 저장 → PushOver에 URL 전달

**D1 변경**:

- [x] `messages` 테이블에 `status = "queued"` 추가
- [x] `messages` 테이블에 `image_url` 컬럼 추가 (R2 이미지 참조)
- [x] `failed_deliveries`에서 메시지 본문 컬럼 제거 (KV로 이관)
- [x] Cron Recovery → KV 기반 본문 복원 로직

### v0.3.0 - Webhook 고도화

- [ ] 웹훅 재시도 정책 (exponential backoff)
- [ ] 웹훅 전송 상태 실시간 모니터링
- [ ] 웹훅 페이로드 검증
- [ ] 다중 수신자 병렬 webhook

### v0.4.0 - 모바일 앱

- [ ] React Native 기반 iOS/Android 앱 (현재 기능 모바일 포팅)
- [ ] Worker API 연동 (메시지 전송/조회)
- [ ] 로컬 알림 관리
- [ ] Dashboard 기능 이식

---

## v2.0 - 독립 알림 시스템

PushOver wrapper가 아닌 자체 알림 플랫폼으로 전환

### 핵심 변경사항

- **PushOver API wrapper → 알림 플랫폼**
- **단일 채널 → 멀티 채널** (PushOver, Slack, Discord, SMS, Email)
- **Proxy → 직접 연동** (APNs/FCM)

### v2.0.0 - APNs/FCM 연동

- [ ] **APNs 직접 연동** (iOS)
  - 인증서/키 관리
  - JWT 토큰 생성
  - Feedback Service 연동
- [ ] **FCM 직접 연동** (Android)
  - Firebase 프로젝트 설정
  - Server Key/OAuth 관리
  - 토픰 등록/관리
- [ ] 알림 전송 큐 (Producer-Consumer 패턴)
- [ ] 우선순위 처리 (Priority Queue)
- [ ] 실패 처리 및 재시도 정책

### v2.1.0 - 멀티 채널

- [ ] **PushOver** (기본 채널)
- [ ] **Slack** webhook
- [ ] **Telegram** Bot API
- [ ] **Discord** webhook
- [ ] **Kakaotalk** 알림톡 API
- [ ] **Email** (SMTP)

### v2.2.0 - 자체 모바일 앱

- [ ] React Native iOS/Android 앱
- [ ] APNs/FCM 토큰 등록
- [ ] 로컬 알림 수신/관리
- [ ] 설정 관리 (채널, 필터)

### v2.3.0 - 고급 기능

- [ ] 알림 템플릿
- [ ] 일괄 스케줄링
- [ ] 수신자 그룹
- [ ] Rate Limiting
- [ ] 알림 통계/대시보드
