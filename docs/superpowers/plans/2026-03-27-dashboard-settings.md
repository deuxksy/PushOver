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

- [ ] **Step 1: WorkerTab 생성**

```typescript
// dashboard/src/app/settings/components/WorkerTab.tsx
'use client';

import { useState, useEffect } from 'react';
import { Settings, DEFAULT_VALUES } from '@/lib/settings';

interface WorkerTabProps {
  settings: Settings;
  onUpdate: (updates: Partial<Settings>) => void;
  onReset: () => void;
}

export function WorkerTab({ settings, onUpdate, onReset }: WorkerTabProps) {
  const [url, setUrl] = useState(settings.worker.url);
  const [webhookSecret, setWebhookSecret] = useState(settings.worker.webhookSecret || '');
  const [showResetConfirm, setShowResetConfirm] = useState(false);

  useEffect(() => {
    setUrl(settings.worker.url);
    setWebhookSecret(settings.worker.webhookSecret || '');
  }, [settings.worker]);

  const handleSave = () => {
    onUpdate({
      worker: { url, webhookSecret: webhookSecret || undefined }
    });
  };

  const handleReset = () => {
    if (showResetConfirm) {
      onReset();
      setUrl(DEFAULT_VALUES.worker.url);
      setWebhookSecret('');
      setShowResetConfirm(false);
    } else {
      setShowResetConfirm(true);
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center gap-3 mb-6">
        <span className="text-2xl">⚙️</span>
        <div>
          <h3 className="font-semibold text-zinc-100">Worker 설정</h3>
          <p className="text-sm text-zinc-400">Cloudflare Worker 구성</p>
        </div>
      </div>

      <div>
        <label className="block text-sm font-medium text-zinc-300 mb-2">
          Worker URL <span className="text-red-400">*</span>
        </label>
        <input
          type="text"
          value={url}
          onChange={(e) => setUrl(e.target.value)}
          placeholder="https://pushover-worker.cromksy.workers.dev"
          className="w-full px-3 py-2 border border-zinc-700 rounded-lg bg-zinc-800 text-zinc-100 font-mono text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
        <p className="text-xs text-zinc-500 mt-1">Cloudflare Worker 배포 URL</p>
      </div>

      <div>
        <label className="block text-sm font-medium text-zinc-300 mb-2">
          Webhook Secret <span className="text-zinc-500">(선택)</span>
        </label>
        <input
          type="password"
          value={webhookSecret}
          onChange={(e) => setWebhookSecret(e.target.value)}
          placeholder="wrangler.toml의 WEBHOOK_SECRET 값"
          className="w-full px-3 py-2 border border-zinc-700 rounded-lg bg-zinc-800 text-zinc-100 font-mono text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
        <p className="text-xs text-zinc-500 mt-1">wrangler.toml 참조</p>
      </div>

      <div className="flex gap-3">
        <button onClick={handleSave} className="px-4 py-2 bg-blue-600 text-white rounded-lg font-medium hover:bg-blue-700 transition-colors">
          저장
        </button>
        <button onClick={handleReset} className={`px-4 py-2 rounded-lg font-medium transition-colors ${showResetConfirm ? 'bg-red-600 text-white hover:bg-red-700' : 'bg-zinc-700 text-zinc-100 hover:bg-zinc-600'}`}>
          {showResetConfirm ? '확인' : '초기화'}
        </button>
        {showResetConfirm && (
          <button onClick={() => setShowResetConfirm(false)} className="px-4 py-2 bg-zinc-700 text-zinc-100 rounded-lg font-medium hover:bg-zinc-600">
            취소
          </button>
        )}
      </div>
    </div>
  );
}
```

- [ ] **Step 2: 커밋**

```bash
git add dashboard/src/app/settings/components/WorkerTab.tsx
git commit -m "feat: add WorkerTab component with reset confirmation"
```

---

## Task 5: Notification 탭 컴포넌트

**Files:**
- Create: `dashboard/src/app/settings/components/NotificationTab.tsx`

- [ ] **Step 1: NotificationTab 생성**

```typescript
// dashboard/src/app/settings/components/NotificationTab.tsx
'use client';

import { useState, useEffect } from 'react';
import { Settings, SOUND_OPTIONS, PRIORITY_OPTIONS } from '@/lib/settings';

interface NotificationTabProps {
  settings: Settings;
  onUpdate: (updates: Partial<Settings>) => void;
  onReset: () => void;
}

export function NotificationTab({ settings, onUpdate, onReset }: NotificationTabProps) {
  const [sound, setSound] = useState(settings.notification.sound);
  const [priority, setPriority] = useState(settings.notification.priority);
  const [showResetConfirm, setShowResetConfirm] = useState(false);

  useEffect(() => {
    setSound(settings.notification.sound);
    setPriority(settings.notification.priority);
  }, [settings.notification]);

  const handleSave = () => {
    onUpdate({
      notification: { sound, device: 'all', priority }
    });
  };

  const handleReset = () => {
    if (showResetConfirm) {
      onReset();
      setSound('pushover');
      setPriority(0);
      setShowResetConfirm(false);
    } else {
      setShowResetConfirm(true);
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center gap-3 mb-6">
        <span className="text-2xl">🔔</span>
        <div>
          <h3 className="font-semibold text-zinc-100">알림 설정</h3>
          <p className="text-sm text-zinc-400">기본 알림 옵션</p>
        </div>
      </div>

      <div>
        <label className="block text-sm font-medium text-zinc-300 mb-2">기본 사운드</label>
        <select value={sound} onChange={(e) => setSound(e.target.value)} className="w-full px-3 py-2 border border-zinc-700 rounded-lg bg-zinc-800 text-zinc-100 focus:outline-none focus:ring-2 focus:ring-blue-500">
          {SOUND_OPTIONS.map((opt) => (
            <option key={opt} value={opt}>{opt === 'pushover' ? '🔔 PushOver (기본)' : opt}</option>
          ))}
        </select>
        <p className="text-xs text-zinc-500 mt-1">기기에서 설정한 사운드가 우선됩니다</p>
      </div>

      <div>
        <label className="block text-sm font-medium text-zinc-300 mb-2">기본 기기</label>
        <select value="all" disabled className="w-full px-3 py-2 border border-zinc-700 rounded-lg bg-zinc-800 text-zinc-100 opacity-60">
          <option value="all">📱 모든 기기</option>
        </select>
        <p className="text-xs text-zinc-500 mt-1">v1에서는 모든 기기만 지원</p>
      </div>

      <div>
        <label className="block text-sm font-medium text-zinc-300 mb-2">우선순위</label>
        <select value={priority} onChange={(e) => setPriority(Number(e.target.value))} className="w-full px-3 py-2 border border-zinc-700 rounded-lg bg-zinc-800 text-zinc-100 focus:outline-none focus:ring-2 focus:ring-blue-500">
          {PRIORITY_OPTIONS.map((opt) => (
            <option key={opt.value} value={opt.value}>{opt.label}</option>
          ))}
        </select>
      </div>

      <div className="flex gap-3">
        <button onClick={handleSave} className="px-4 py-2 bg-amber-600 text-white rounded-lg font-medium hover:bg-amber-700 transition-colors">
          저장
        </button>
        <button onClick={handleReset} className={`px-4 py-2 rounded-lg font-medium transition-colors ${showResetConfirm ? 'bg-red-600 text-white hover:bg-red-700' : 'bg-zinc-700 text-zinc-100 hover:bg-zinc-600'}`}>
          {showResetConfirm ? '확인' : '초기화'}
        </button>
        {showResetConfirm && (
          <button onClick={() => setShowResetConfirm(false)} className="px-4 py-2 bg-zinc-700 text-zinc-100 rounded-lg font-medium hover:bg-zinc-600">
            취소
          </button>
        )}
      </div>
    </div>
  );
}
```

- [ ] **Step 2: 커밋**

```bash
git add dashboard/src/app/settings/components/NotificationTab.tsx
git commit -m "feat: add NotificationTab component with sound/priority options"
```

---

## Task 6: Settings 페이지 리팩토링

**Files:**
- Modify: `dashboard/src/app/settings/page.tsx`

- [ ] **Step 1: 3개 탭 구조로 리팩토링**

```typescript
// dashboard/src/app/settings/page.tsx
'use client';

import { useState } from 'react';
import Link from 'next/link';
import { useSettings } from './hooks/useSettings';
import { PushOverTab } from './components/PushOverTab';
import { WorkerTab } from './components/WorkerTab';
import { NotificationTab } from './components/NotificationTab';

type TabType = 'pushover' | 'worker' | 'notification';

const TABS: { id: TabType; label: string; color: string }[] = [
  { id: 'pushover', label: 'PushOver', color: 'blue' },
  { id: 'worker', label: 'Worker', color: 'green' },
  { id: 'notification', label: '알림', color: 'amber' }
];

export default function SettingsPage() {
  const [activeTab, setActiveTab] = useState<TabType>('pushover');
  const { settings, isLoading, error, updateSettings, resetTab } = useSettings();

  if (isLoading) {
    return (
      <div className="min-h-screen bg-zinc-950 p-8">
        <div className="animate-pulse space-y-4 max-w-2xl mx-auto">
          <div className="h-10 bg-zinc-800 rounded" />
          <div className="h-10 bg-zinc-800 rounded" />
          <div className="h-10 bg-zinc-800 rounded w-1/2" />
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-zinc-950">
      <nav className="border-b border-zinc-800 bg-zinc-950">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex h-16 items-center justify-between">
            <h1 className="text-xl font-bold text-zinc-100">Settings</h1>
            <Link href="/" className="px-4 py-2 text-sm font-medium text-zinc-400 hover:text-zinc-100">
              ← Home
            </Link>
          </div>
        </div>
      </nav>

      <main className="max-w-2xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <div className="flex gap-1 border-b border-zinc-800 mb-6">
          {TABS.map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`px-4 py-3 text-sm font-medium rounded-t-lg transition-colors ${
                activeTab === tab.id
                  ? tab.color === 'blue' ? 'bg-blue-600 text-white'
                    : tab.color === 'green' ? 'bg-green-600 text-white'
                    : 'bg-amber-600 text-white'
                  : 'text-zinc-400 hover:text-zinc-100'
              }`}
            >
              {tab.label}
            </button>
          ))}
        </div>

        {error && (
          <div className="mb-4 p-3 bg-red-900/30 border border-red-700 rounded-lg">
            <p className="text-sm text-red-400">{error}</p>
          </div>
        )}

        <div className="bg-zinc-900 border border-zinc-800 rounded-lg p-6">
          {activeTab === 'pushover' && <PushOverTab settings={settings} onUpdate={updateSettings} />}
          {activeTab === 'worker' && <WorkerTab settings={settings} onUpdate={updateSettings} onReset={() => resetTab('worker')} />}
          {activeTab === 'notification' && <NotificationTab settings={settings} onUpdate={updateSettings} onReset={() => resetTab('notification')} />}
        </div>
      </main>
    </div>
  );
}
```

- [ ] **Step 2: 커밋**

```bash
git add dashboard/src/app/settings/page.tsx
git commit -m "feat: refactor settings page to 3-tab structure"
```

---

## Task 7: API 클라이언트 수정

**Files:**
- Modify: `dashboard/src/lib/api.ts`

- [ ] **Step 1: PushOver body auth 방식으로 변경**

```typescript
// dashboard/src/lib/api.ts
import { Settings, DEFAULT_VALUES, loadSettings } from './settings';

export interface Message {
  id: string;
  message: string;
  title?: string;
  status: string;
  sent_at: string;
  delivered_at?: string;
  acknowledged_at?: string;
}

export interface SendMessageRequest {
  message: string;
  title?: string;
  priority?: number;
  sound?: string;
  device?: string;
  url?: string;
  url_title?: string;
  html?: boolean;
}

export interface SendMessageResponse {
  status: number;
  request: string;
  receipt?: string;
}

export class PushOverAPI {
  private settings: Settings;

  constructor() {
    const stored = loadSettings();
    this.settings = stored ?? {
      pushover: { ...DEFAULT_VALUES.pushover },
      worker: { ...DEFAULT_VALUES.worker },
      notification: { ...DEFAULT_VALUES.notification }
    };
  }

  private async request<T>(endpoint: string, options?: RequestInit): Promise<T> {
    const workerUrl = this.settings.worker?.url || DEFAULT_VALUES.worker.url;

    const response = await fetch(`${workerUrl}${endpoint}`, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        ...(this.settings.worker?.webhookSecret && {
          'X-Webhook-Secret': this.settings.worker.webhookSecret
        }),
        ...options?.headers,
      },
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({ message: 'Unknown error' }));
      throw new Error(error.message || `API Error: ${response.status}`);
    }

    return response.json();
  }

  async getHistory(limit = 50): Promise<Message[]> {
    return this.request<Message[]>(`/api/v1/messages?limit=${limit}`);
  }

  async sendMessage(data: SendMessageRequest): Promise<SendMessageResponse> {
    // 필수 설정 검증
    if (!this.settings.pushover?.apiToken || !this.settings.pushover?.userKey) {
      throw new Error('PushOver credentials not configured');
    }

    // PushOver API는 token/user를 body에 포함
    return this.request('/api/v1/messages', {
      method: 'POST',
      body: JSON.stringify({
        token: this.settings.pushover.apiToken,
        user: this.settings.pushover.userKey,
        message: data.message,
        title: data.title,
        sound: data.sound || this.settings.notification?.sound || DEFAULT_VALUES.notification.sound,
        device: data.device || this.settings.notification?.device || DEFAULT_VALUES.notification.device,
        priority: data.priority ?? this.settings.notification?.priority ?? DEFAULT_VALUES.notification.priority,
        url: data.url,
        url_title: data.url_title,
        html: data.html,
      }),
    });
  }

  async getStatus(receipt: string): Promise<{ status: string; acknowledged: boolean }> {
    return this.request(`/api/v1/messages/${receipt}/status`);
  }
}

export const pushOverAPI = new PushOverAPI();
```

- [ ] **Step 2: 커밋**

```bash
git add dashboard/src/lib/api.ts
git commit -m "feat: update API client to use body auth for PushOver"
```

---

## Task 8: 홈페이지 연동

**Files:**
- Modify: `dashboard/src/app/page.tsx`

- [ ] **Step 1: 설정 미설치 배너 추가**

```typescript
// dashboard/src/app/page.tsx
'use client';

import { useEffect, useState } from 'react';
import Link from 'next/link';
import { loadSettings } from '@/lib/settings';

export default function HomePage() {
  const [showBanner, setShowBanner] = useState(false);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    const settings = loadSettings();
    if (!settings?.pushover?.apiToken || !settings?.pushover?.userKey) {
      setShowBanner(true);
    }
    setIsLoading(false);
  }, []);

  if (isLoading) {
    return (
      <div className="min-h-screen bg-zinc-950 p-8">
        <div className="animate-pulse space-y-4 max-w-4xl mx-auto">
          <div className="h-12 bg-zinc-800 rounded" />
          <div className="h-32 bg-zinc-800 rounded" />
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-zinc-950">
      <main className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {showBanner && (
          <div className="bg-amber-900/30 border border-amber-700 rounded-lg p-4 mb-6">
            <div className="flex items-center gap-3">
              <span className="text-2xl">⚠️</span>
              <div className="flex-1">
                <p className="font-medium text-amber-200">PushOver 설정이 필요합니다</p>
                <p className="text-sm text-zinc-400">알림을 받으려면 Settings 페이지에서 API Token과 User Key를 설정해주세요.</p>
              </div>
              <Link href="/settings" className="px-4 py-2 bg-amber-600 text-white rounded-lg font-medium hover:bg-amber-700 transition-colors whitespace-nowrap">
                설정하기
              </Link>
            </div>
          </div>
        )}

        <h1 className="text-3xl font-bold text-zinc-100 mb-6">PushOver Dashboard</h1>
        <p className="text-zinc-400">PushOver 알림을 관리하는 대시보드입니다.</p>
      </main>
    </div>
  );
}
```

- [ ] **Step 2: 커밋**

```bash
git add dashboard/src/app/page.tsx
git commit -m "feat: add settings warning banner to homepage"
```

---

## Task 9: 통합 테스트

- [ ] **Step 1: 개발 서버 실행**

```bash
cd dashboard && pnpm dev
```

- [ ] **Step 2: Settings 페이지 접근 및 탭 전환 테스트**

브라우저에서 http://localhost:3000/settings 접속
- PushOver, Worker, 알림 탭 클릭 시 정상 전환 확인
- 각 탭의 입력 필드 정상 표시 확인

- [ ] **Step 3: localStorage 저장 테스트**

1. PushOver 탭에서 API Token, User Key 입력 후 저장
2. 브라우저 개발자 도구 → Application → Local Storage 확인
3. `pushover-settings` 키에 base64 인코딩된 값 존재 확인

- [ ] **Step 4: PushOver 테스트 알림 발송**

1. PushOver 탭에서 테스트 버튼 클릭
2. PushOver 앱에서 알림 수신 확인
3. 실패 시 에러 메시지 정상 표시 확인

- [ ] **Step 5: 홈페이지 배너 테스트**

1. localStorage 삭제 후 홈페이지 접속
2. "PushOver 설정이 필요합니다" 배너 표시 확인
3. "설정하기" 버튼 클릭 시 /settings 이동 확인

- [ ] **Step 6: 초기화 기능 테스트**

1. Worker/알림 탭에서 값 변경 후 저장
2. 초기화 버튼 클릭 → 확인 다이얼로그 → 확인
3. 기본값으로 복원 확인

- [ ] **Step 7: 커밋**

```bash
git add -A
git commit -m "feat: complete Dashboard Settings implementation"
```

---

**Plan complete and saved to `docs/superpowers/plans/2026-03-27-dashboard-settings.md`.**

**Two execution options:**

1. **Subagent-Driven (recommended)** - I dispatch a fresh subagent per task, review between tasks, fast iteration

2. **Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints

**Which approach?**
