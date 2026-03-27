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
| 기기 | select | X | v1: 'all' 고정 |
| 우선순위 | select | X | -2 ~ 2 |

> **v1 제한사항**: 기기 선택은 'all'만 지원. 동적 기기 로드는 v2에서 PushOver API `/1/devices.json` 연동 후 구현 예정.

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
    apiToken: string;      // 필수
    userKey: string;       // 필수
  };
  worker: {
    url: string;           // 필수
    webhookSecret?: string; // 선택
  };
  notification: {
    sound?: string;        // 기본값: 'pushover'
    device?: string;       // 기본값: 'all' (v1에서는 'all'만 지원)
    priority?: number;     // 기본값: 0
  };
}
```

### 기본값 (초기화 시 사용)

| 탭 | 필드 | 기본값 |
|---|---|---|
| PushOver | apiToken | `''` (빈 문자열) |
| PushOver | userKey | `''` (빈 문자열) |
| Worker | url | `'https://pushover-worker.cromksy.workers.dev'` |
| Worker | webhookSecret | `undefined` |
| 알림 | sound | `'pushover'` |
| 알림 | device | `'all'` |
| 알림 | priority | `0` |

> **초기화 범위**: 초기화 버튼은 현재 활성 탭의 설정만 기본값으로 복원합니다. 다른 탭의 설정은 유지됩니다.

### 저장 방식
- 키: `pushover-settings`
- 인코딩: base64 인코딩 (난독화 목적, **암호화 아님**)
- 검증: 저장 전 필수 필드 체크

> ⚠️ **보안 고지**: base64는 인코딩이며 실제 암호화가 아닙니다. 브라우저 개발자 도구에서 localStorage 내용을 볼 수 있습니다. 민감한 정보 보호를 위해서는 향후 AES 암호화 도입 필요. v1에서는 사용자 책임 하에 사용.

### useSettings Hook 명세

```typescript
// src/app/settings/hooks/useSettings.ts
interface UseSettingsReturn {
  settings: Settings | null;
  isLoading: boolean;
  error: string | null;
  saveSettings: (newSettings: Partial<Settings>) => Promise<void>;
  resetTab: (tab: 'pushover' | 'worker' | 'notification') => void;
}

function useSettings(): UseSettingsReturn {
  // 상태
  const [settings, setSettings] = useState<Settings | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // 초기 로드
  useEffect(() => {
    try {
      const stored = localStorage.getItem('pushover-settings');
      if (stored) {
        const decoded = JSON.parse(atob(stored));
        setSettings(decoded);
      }
    } catch (e) {
      setError('설정을 불러오는데 실패했습니다');
      console.error('Settings load error:', e);
    } finally {
      setIsLoading(false);
    }
  }, []);

  // 저장
  const saveSettings = async (newSettings: Partial<Settings>) => {
    try {
      const merged = { ...settings, ...newSettings };
      localStorage.setItem('pushover-settings', btoa(JSON.stringify(merged)));
      setSettings(merged);
      setError(null);
    } catch (e) {
      setError('설정 저장에 실패했습니다');
      throw e;
    }
  };

  // 탭별 초기화
  const resetTab = (tab: string) => {
    const defaults = { /* 위 기본값 테이블 참조 */ };
    saveSettings({ [tab]: defaults[tab] });
  };

  return { settings, isLoading, error, saveSettings, resetTab };
}
```

---

## API 연동

### 메시지 전송

```typescript
// src/lib/api.ts
interface MessageInput {
  text: string;
  title?: string;
}

interface SendMessageResponse {
  status: number;  // 1 = success
  request: string; // unique request ID
  receipt?: string; // for emergency priority (priority=2)
}

async function sendMessage(message: MessageInput): Promise<SendMessageResponse> {
  const settings = loadSettings();

  // 필수 설정 검증
  if (!settings?.pushover?.apiToken || !settings?.pushover?.userKey) {
    throw new Error('PushOver credentials not configured');
  }

  // Worker URL 기본값 설정
  const workerUrl = settings.worker?.url || 'https://pushover-worker.cromksy.workers.dev';

  const response = await fetch(`${workerUrl}/api/v1/messages`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      // Webhook Secret이 있으면 추가 (선택)
      ...(settings.worker?.webhookSecret && {
        'X-Webhook-Secret': settings.worker.webhookSecret
      }),
    },
    body: JSON.stringify({
      // PushOver API 필수 필드
      token: settings.pushover.apiToken,
      user: settings.pushover.userKey,
      message: message.text,
      // 선택 필드
      title: message.title,
      sound: settings.notification?.sound || 'pushover',
      device: settings.notification?.device || 'all',
      priority: settings.notification?.priority ?? 0,
    }),
  });

  // 에러 처리
  if (!response.ok) {
    const error = await response.json().catch(() => ({ message: 'Unknown error' }));
    throw new Error(error.message || `API request failed: ${response.status}`);
  }

  return response.json();
}
```

> **참고**: PushOver API는 `Authorization` 헤더가 아닌 요청 본문에 `token`과 `user`를 포함합니다.

---

## UI/UX

### 스타일
- 다크모드 기본 (기존 대시보드와 일치)
- Tailwind CSS 사용
- zinc 색상 팔레트

### 버튼
- 저장: 파란색 (bg-blue-600) - 현재 탭 설정 저장
- 초기화: 회색 (bg-zinc-700) - 기본값으로 복원 (확인 다이얼로그 표시)
- 테스트: 초록색 (bg-green-600) - PushOver 탭에서만 표시

### 초기화 버튼 동작
1. 확인 다이얼로그: "설정을 기본값으로 초기화하시겠습니까?"
2. 현재 탭 설정만 기본값으로 복원
3. localStorage 업데이트
4. 토스트 메시지: "설정이 초기화되었습니다"

### 피드백
- 저장 성공: 토스트 메시지
- 저장 실패: 에러 메시지 표시
- 테스트 성공: PushOver 알림 발송 확인

### 테스트 버튼 에러 처리

```typescript
async function handleTestNotification() {
  setIsTesting(true);
  setTestError(null);

  try {
    // 필수 필드 검증
    if (!apiToken || !userKey) {
      throw new Error('API Token과 User Key를 입력해주세요');
    }

    await sendMessage({
      text: '테스트 알림 - PushOver 설정이 완료되었습니다!',
      title: 'PushOver 테스트'
    });

    setTestSuccess(true);
    setTimeout(() => setTestSuccess(false), 3000);
  } catch (error) {
    // 네트워크 에러
    if (error instanceof TypeError && error.message.includes('fetch')) {
      setTestError('네트워크 연결을 확인해주세요');
    }
    // API 에러
    else if (error instanceof Error) {
      setTestError(error.message);
    }
    // 알 수 없는 에러
    else {
      setTestError('알 수 없는 오류가 발생했습니다');
    }
  } finally {
    setIsTesting(false);
  }
}
```

---

## 홈페이지 연동

### 설정 미설치 시 안내

홈페이지(`/`)에서 PushOver 설정이 없는 경우 안내 배너 표시:

```tsx
// src/app/page.tsx
function HomePage() {
  const { settings, isLoading } = useSettings();
  const [showBanner, setShowBanner] = useState(false);

  useEffect(() => {
    if (!isLoading && (!settings?.pushover?.apiToken || !settings?.pushover?.userKey)) {
      setShowBanner(true);
    }
  }, [settings, isLoading]);

  if (showBanner) {
    return (
      <div className="bg-amber-900/30 border border-amber-700 rounded-lg p-4 mb-6">
        <div className="flex items-center gap-3">
          <span className="text-2xl">⚠️</span>
          <div>
            <p className="font-medium">PushOver 설정이 필요합니다</p>
            <p className="text-sm text-zinc-400">
              알림을 받으려면 Settings 페이지에서 API Token과 User Key를 설정해주세요.
            </p>
          </div>
          <Link href="/settings" className="ml-auto px-4 py-2 bg-amber-600 rounded-lg">
            설정하기
          </Link>
        </div>
      </div>
    );
  }

  // ... 기본 홈페이지 콘텐츠
}
```

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
