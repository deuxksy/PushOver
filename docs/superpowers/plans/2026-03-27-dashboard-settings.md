# Dashboard Settings Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 3개 탭 구조의 Settings 페이지 구현 (PushOver, Worker, 알림 설정)

**Architecture:** localStorage에 base64 인코딩으로 설정 저장, useSettings 커스텀 훅으로 상태 관리, 각 탭은 독립 컴포넌트로 분리

**Tech Stack:** Next.js 16, React 19, TypeScript, Tailwind CSS 4

---

## File Structure

```
dashboard/src/
├── app/
│   ├── page.tsx                    # 수정: 설정 미설치 시 배너 추가
│   └── settings/
│       ├── page.tsx                # 수정: 3개 탭 구조로 변경
│       ├── components/
│       │   ├── PushOverTab.tsx     # 생성
│       │   ├── WorkerTab.tsx       # 생성
│       │   └── NotificationTab.tsx # 생성
│       └── hooks/
│           └── useSettings.ts      # 생성
└── lib/
    ├── api.ts                      # 수정
    └── settings.ts                 # 생성
```

---

## Task 1: Settings 타입 및 상수 정의

**Files:**
- Create: `dashboard/src/lib/settings.ts`

- [ ] **Step 1: Settings 타입 및 상수 파일 생성**

```typescript
// dashboard/src/lib/settings.ts

export interface Settings {
  pushover: {
    apiToken: string;
    userKey: string;
  };
  worker: {
    url: string;
    webhookSecret?: string;
  };
  notification: {
    sound: string;
    device: string;
    priority: number;
  };
}

export const DEFAULT_VALUES: Settings = {
  pushover: { apiToken: '', userKey: '' },
  worker: { url: 'https://pushover-worker.cromksy.workers.dev' },
  notification: { sound: 'pushover', device: 'all', priority: 0 }
};

export const SOUND_OPTIONS = [
  'pushover', 'bike', 'bugle', 'cashregister', 'classical', 'cosmic',
  'falling', 'gamelan', 'incoming', 'intermission', 'magic', 'mechanical',
  'pianobar', 'siren', 'spacealarm', 'tugboat', 'alien', 'climb',
  'persistent', 'echo', 'updown', 'vibrate', 'none'
] as const;

export const PRIORITY_OPTIONS = [
  { value: -2, label: '최저 (방해 금지 시 무음)' },
  { value: -1, label: '낮음 (소리 없이 배지만)' },
  { value: 0, label: '보통 (기본)' },
  { value: 1, label: '높음 (방해 금지 무시)' },
  { value: 2, label: '긴급 (확인 시까지 반복)' }
] as const;

export const SETTINGS_STORAGE_KEY = 'pushover-settings';

export function loadSettings(): Settings | null {
  if (typeof window === 'undefined') return null;
  try {
    const stored = localStorage.getItem(SETTINGS_STORAGE_KEY);
    if (stored) {
      return JSON.parse(atob(stored));
    }
  } catch (e) {
    console.error('Settings load error:', e);
  }
  return null;
}

export function saveSettings(settings: Settings): void {
  localStorage.setItem(SETTINGS_STORAGE_KEY, btoa(JSON.stringify(settings)));
}

export function validateSettings(settings: Partial<Settings>): string[] {
  const errors: string[] = [];

  if (settings.pushover?.apiToken && !/^[a-zA-Z0-9]{30,}$/.test(settings.pushover.apiToken)) {
    errors.push('API Token 형식이 올바르지 않습니다');
  }
  if (settings.pushover?.userKey && !/^[a-zA-Z0-9]{30,}$/.test(settings.pushover.userKey)) {
    errors.push('User Key 형식이 올바르지 않습니다');
  }
  if (settings.worker?.url) {
    try {
      new URL(settings.worker.url);
    } catch {
      errors.push('Worker URL 형식이 올바르지 않습니다');
    }
  }

  return errors;
}
```

- [ ] **Step 2: 커밋**

```bash
git add dashboard/src/lib/settings.ts
git commit -m "feat: add Settings types and constants"
```

---

## Task 2: useSettings Hook 구현

**Files:**
- Create: `dashboard/src/app/settings/hooks/useSettings.ts`

- [ ] **Step 1: useSettings Hook 생성**

```typescript
// dashboard/src/app/settings/hooks/useSettings.ts
'use client';

import { useState, useEffect, useCallback } from 'react';
import {
  Settings,
  DEFAULT_VALUES,
  loadSettings,
  saveSettings
} from '@/lib/settings';

interface UseSettingsReturn {
  settings: Settings;
  isLoading: boolean;
  error: string | null;
  updateSettings: (updates: Partial<Settings>) => void;
  resetTab: (tab: keyof Settings) => void;
  hasRequiredSettings: boolean;
}

export function useSettings(): UseSettingsReturn {
  const [settings, setSettings] = useState<Settings>(DEFAULT_VALUES);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const stored = loadSettings();
    if (stored) {
      setSettings(stored);
    }
    setIsLoading(false);
  }, []);

  const updateSettings = useCallback((updates: Partial<Settings>) => {
    setSettings(prev => {
      const merged = { ...prev };
      if (updates.pushover) {
        merged.pushover = { ...prev.pushover, ...updates.pushover };
      }
      if (updates.worker) {
        merged.worker = { ...prev.worker, ...updates.worker };
      }
      if (updates.notification) {
        merged.notification = { ...prev.notification, ...updates.notification };
      }
      try {
        saveSettings(merged);
        setError(null);
      } catch (e) {
        setError('설정 저장에 실패했습니다');
      }
      return merged;
    });
  }, []);

  const resetTab = useCallback((tab: keyof Settings) => {
    updateSettings({ [tab]: DEFAULT_VALUES[tab] });
  }, [updateSettings]);

  const hasRequiredSettings = Boolean(
    settings.pushover.apiToken && settings.pushover.userKey
  );

  return { settings, isLoading, error, updateSettings, resetTab, hasRequiredSettings };
}
```

- [ ] **Step 2: 커밋**

```bash
git add dashboard/src/app/settings/hooks/useSettings.ts
git commit -m "feat: add useSettings hook"
```

---

## Task 3: PushOver 탭 컴포넌트

**Files:**
- Create: `dashboard/src/app/settings/components/PushOverTab.tsx`

- [ ] **Step 1: PushOverTab 생성**

```typescript
// dashboard/src/app/settings/components/PushOverTab.tsx
'use client';

import { useState, useEffect } from 'react';
import { Settings } from '@/lib/settings';

interface PushOverTabProps {
  settings: Settings;
  onUpdate: (updates: Partial<Settings>) => void;
}

export function PushOverTab({ settings, onUpdate }: PushOverTabProps) {
  const [apiToken, setApiToken] = useState(settings.pushover.apiToken);
  const [userKey, setUserKey] = useState(settings.pushover.userKey);
  const [isTesting, setIsTesting] = useState(false);
  const [testResult, setTestResult] = useState<{ success: boolean; message: string } | null>(null);

  useEffect(() => {
    setApiToken(settings.pushover.apiToken);
    setUserKey(settings.pushover.userKey);
  }, [settings.pushover]);

  const handleSave = () => {
    onUpdate({ pushover: { apiToken, userKey } });
  };

  const handleTest = async () => {
    if (!apiToken || !userKey) {
      setTestResult({ success: false, message: 'API Token과 User Key를 입력해주세요' });
      return;
    }

    setIsTesting(true);
    setTestResult(null);

    try {
      const response = await fetch(`${settings.worker.url}/api/v1/messages`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          token: apiToken,
          user: userKey,
          message: '테스트 알림 - PushOver 설정이 완료되었습니다!',
          title: 'PushOver 테스트',
          sound: settings.notification.sound,
          priority: settings.notification.priority
        })
      });

      if (response.ok) {
        setTestResult({ success: true, message: '테스트 알림이 발송되었습니다!' });
      } else {
        const error = await response.json().catch(() => ({ message: 'Unknown error' }));
        setTestResult({ success: false, message: error.message || `Error: ${response.status}` });
      }
    } catch (error) {
      setTestResult({ success: false, message: '네트워크 연결을 확인해주세요' });
    } finally {
      setIsTesting(false);
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center gap-3 mb-6">
        <span className="text-2xl">📱</span>
        <div>
          <h3 className="font-semibold text-zinc-100">PushOver 설정</h3>
          <p className="text-sm text-zinc-400">PushOver API 인증 정보</p>
        </div>
      </div>

      <div>
        <label className="block text-sm font-medium text-zinc-300 mb-2">
          API Token <span className="text-red-400">*</span>
        </label>
        <input
          type="password"
          value={apiToken}
          onChange={(e) => setApiToken(e.target.value)}
          placeholder="azGDORePK8gMaC0QOYAMyEL..."
          className="w-full px-3 py-2 border border-zinc-700 rounded-lg bg-zinc-800 text-zinc-100 font-mono text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
        <p className="text-xs text-zinc-500 mt-1">pushover.net/apps 에서 확인</p>
      </div>

      <div>
        <label className="block text-sm font-medium text-zinc-300 mb-2">
          User Key <span className="text-red-400">*</span>
        </label>
        <input
          type="password"
          value={userKey}
          onChange={(e) => setUserKey(e.target.value)}
          placeholder="uQiRzpo4DXghDmr9QzzfQu27cmVRsG..."
          className="w-full px-3 py-2 border border-zinc-700 rounded-lg bg-zinc-800 text-zinc-100 font-mono text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
        <p className="text-xs text-zinc-500 mt-1">pushover.net 메인 페이지 상단</p>
      </div>

      {testResult && (
        <div className={`p-3 rounded-lg ${testResult.success ? 'bg-green-900/30 border border-green-700' : 'bg-red-900/30 border border-red-700'}`}>
          <p className={`text-sm ${testResult.success ? 'text-green-400' : 'text-red-400'}`}>
            {testResult.message}
          </p>
        </div>
      )}

      <div className="flex gap-3">
        <button onClick={handleSave} className="px-4 py-2 bg-blue-600 text-white rounded-lg font-medium hover:bg-blue-700 transition-colors">
          저장
        </button>
        <button onClick={handleTest} disabled={isTesting} className="px-4 py-2 bg-green-600 text-white rounded-lg font-medium hover:bg-green-700 transition-colors disabled:opacity-50">
          {isTesting ? '테스트 중...' : '테스트'}
        </button>
      </div>
    </div>
  );
}
```

- [ ] **Step 2: 커밋**

```bash
git add dashboard/src/app/settings/components/PushOverTab.tsx
git commit -m "feat: add PushOverTab component"
```

---

## Task 4: Worker 탭 컴포넌트

**Files:**
- Create: `dashboard/src/app/settings/components/WorkerTab.tsx`

- [ ] **Step 1: WorkerTab 생성** (코드는 spec 참조)

- [ ] **Step 2: 커밋**

---

## Task 5: Notification 탭 컴포넌트

**Files:**
- Create: `dashboard/src/app/settings/components/NotificationTab.tsx`

- [ ] **Step 1: NotificationTab 생성** (코드는 spec 참조)

- [ ] **Step 2: 커밋**

---

## Task 6: Settings 페이지 리팩토링

**Files:**
- Modify: `dashboard/src/app/settings/page.tsx`

- [ ] **Step 1: 3개 탭 구조로 리팩토링** (코드는 spec 참조)

- [ ] **Step 2: 커밋**

---

## Task 7: API 클라이언트 수정

**Files:**
- Modify: `dashboard/src/lib/api.ts`

- [ ] **Step 1: PushOver body auth 방식으로 변경** (코드는 spec 참조)

- [ ] **Step 2: 커밋**

---

## Task 8: 홈페이지 연동

**Files:**
- Modify: `dashboard/src/app/page.tsx`

- [ ] **Step 1: 설정 미설치 배너 추가** (코드는 spec 참조)

- [ ] **Step 2: 커밋**

---

## Task 9: 통합 테스트

- [ ] **Step 1: 개발 서버 실행 및 기능 테스트**

---

**Plan complete and saved to `docs/superpowers/plans/2026-03-27-dashboard-settings.md`.**

**Two execution options:**

1. **Subagent-Driven (recommended)** - I dispatch a fresh subagent per task, review between tasks, fast iteration

2. **Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints

**Which approach?**
