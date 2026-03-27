# Dashboard Settings 페이지 디자인

**날짜:** 2026-03-27
**상태:** 승인됨

---

## 개요

PushOver Dashboard의 Settings 페이지를 3개 탭 구조로 구현하여 PushOver API 설정, Worker 설정, 알림 설정을 분리 관리한다.

---

## 아키텍처

### 탭 구조

```
┌─────────────────────────────────────────────┐
│  [PushOver]    [Worker]    [알림]           │
├─────────────────────────────────────────────┤
│                                             │
│  탭별 설정 폼                                │
│                                             │
│  [저장]  [초기화]                            │
└─────────────────────────────────────────────┘
```

### 컴포넌트 구조

```
src/app/settings/
├── page.tsx              # 메인 페이지 (탭 상태 관리)
├── components/
│   ├── PushOverTab.tsx   # PushOver 설정 탭
│   ├── WorkerTab.tsx     # Worker 설정 탭
│   └── NotificationTab.tsx # 알림 설정 탭
└── hooks/
    └── useSettings.ts    # localStorage 저장/불러오기
```

---

## 탭별 상세

### 1. PushOver 탭

| 필드 | 타입 | 필수 | 설명 |
|---|---|---|---|
| API Token | text (password) | O | pushover.net/apps |
| User Key | text (password) | O | pushover.net 메인 |

### 2. Worker 탭

| 필드 | 타입 | 필수 | 설명 |
|---|---|---|---|
| Worker URL | text | O | `https://pushover-worker.cromksy.workers.dev` |
| Webhook Secret | text (password) | X | wrangler.toml 참조 |

### 3. 알림 탭

| 필드 | 타입 | 필수 | 옵션 |
|---|---|---|---|
| 사운드 | select | X | 23개 (pushover, bike, bugle, ...) |
| 기기 | select | X | all / 동적 로드 |
| 우선순위 | select | X | -2 ~ 2 |

#### 사운드 옵션 (23개)
```
pushover, bike, bugle, cashregister, classical, cosmic,
falling, gamelan, incoming, intermission, magic, mechanical,
pianobar, siren, spacealarm, tugboat, alien, climb,
persistent, echo, updown, vibrate, none
```

#### 우선순위 옵션
| 값 | 이름 | 설명 |
|---|---|---|
| -2 | 최저 | 방해 금지 시 무음 |
| -1 | 낮음 | 소리 없이 배지만 |
| 0 | 보통 | 기본값 |
| 1 | 높음 | 방해 금지 무시 |
| 2 | 긴급 | 확인 시까지 반복 |

---

## 데이터 저장

### localStorage 키

```typescript
interface Settings {
  pushover: {
    apiToken: string;
    userKey: string;
  };
  worker: {
    url: string;
    webhookSecret: string;
  };
  notification: {
    sound: string;
    device: string;
    priority: number;
  };
}
```

### 저장 방식
- 키: `pushover-settings`
- 암호화: base64 인코딩 (간단한 난독화)
- 검증: 저장 전 필수 필드 체크

---

## API 연동

### 메시지 전송

```typescript
// src/lib/api.ts
async function sendMessage(message: MessageInput) {
  const settings = loadSettings();

  const response = await fetch(`${settings.worker.url}/api/v1/messages`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${settings.pushover.apiToken}`,
    },
    body: JSON.stringify({
      token: settings.pushover.apiToken,
      user: settings.pushover.userKey,
      message: message.text,
      title: message.title,
      sound: settings.notification.sound,
      device: settings.notification.device,
      priority: settings.notification.priority,
    }),
  });

  return response.json();
}
```

---

## UI/UX

### 스타일
- 다크모드 기본 (기존 대시보드와 일치)
- Tailwind CSS 사용
- zinc 색상 팔레트

### 버튼
- 저장: 파란색 (bg-blue-600)
- 초기화: 회색 (bg-zinc-700)
- 테스트: 초록색 (bg-green-600)

### 피드백
- 저장 성공: 토스트 메시지
- 저장 실패: 에러 메시지 표시
- 테스트 성공: PushOver 알림 발송 확인

---

## 성공 기준

1. [ ] 3개 탭이 정상 동작
2. [ ] localStorage 저장/불러오기 동작
3. [ ] PushOver API로 테스트 알림 발송 성공
4. [ ] 설정 없을 때 홈페이지에서 안내 표시
5. [ ] 다크모드 정상 표시

---

## 구현 범위

### 포함
- Settings 페이지 3개 탭
- localStorage 저장/불러오기
- API 연동 (메시지 전송)
- 테스트 버튼

### 제외 (향후 고려)
- 설정값 암호화 (AES)
- 다중 프로필
- 설정 동기화 (Cloudflare D1)
