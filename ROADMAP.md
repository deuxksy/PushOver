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

### v0.2.0 - Queue & KV 활용

- [ ] **Queues**: 메시지 큐 처리 (비동기 전송)
- [ ] **KV**: 캐시 최적화 (메시지, 토큰, 웹훅)
- [ ] 메시지 대기열 처리
- [ ] 실패 메시지 Queue 기반 재시도

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
