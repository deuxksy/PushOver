# Apple Design System Dashboard Redesign — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** PushOver Dashboard 3페이지를 Apple 공식 웹사이트 디자인 언어로 전면 리디자인

**Architecture:** Black Hero → Light Content 교차 섹션 패턴. 공유 GlassNav 컴포넌트 + Tailwind v4 `@theme inline` 토큰. 기존 비즈니스 로직(`lib/`, `hooks/`) 변경 없이 UI만 교체.

**Tech Stack:** Next.js 16.2 (App Router), React 19, Tailwind CSS v4.2 (`@theme inline`), TypeScript, Playwright E2E

**Spec:** `docs/2026-04-06-apple-redesign-design.md`

---

## File Structure

| 작업 | 파일 | 역할 |
|------|------|------|
| Create | `src/components/GlassNav.tsx` | 공유 Glass Navigation (sticky, backdrop-blur) |
| Modify | `src/app/globals.css` | Apple 디자인 토큰 정의 |
| Modify | `src/app/layout.tsx` | System font, GlassNav 포함, `<main pt-12>` |
| Modify | `src/app/page.tsx` | Home: Black Hero → Light Stats + Apple 모달 |
| Modify | `src/app/history/page.tsx` | History: Black Hero → Light Table |
| Modify | `src/app/settings/page.tsx` | Settings: Black Hero → Light Tab Form |
| Modify | `src/app/settings/components/PushOverTab.tsx` | Apple 폼 필드 + Pill 버튼 |
| Modify | `src/app/settings/components/WorkerTab.tsx` | Apple 폼 필드 + Pill 버튼 |
| Modify | `src/app/settings/components/NotificationTab.tsx` | Apple 폼 필드 + Pill 버튼 |
| Modify | `tests/loc.spec.ts` | 새 선택자에 맞게 E2E 테스트 업데이트 |
| No Change | `src/lib/api.ts` | API 클라이언트 — 변경 없음 |
| No Change | `src/lib/settings.ts` | Settings 유틸 — 변경 없음 |
| No Change | `src/app/settings/hooks/useSettings.ts` | 훅 — 변경 없음 |

---

## Task 1: Design Tokens — globals.css

**Files:**
- Modify: `dashboard/src/app/globals.css`

Apple 디자인 토큰을 Tailwind v4 `@theme inline`에 정의. 기존 Geist 폰트 변수 제거, System Font Stack 적용.

- [ ] **Step 1: globals.css 전체 교체**

```css
@import "tailwindcss";

/* Apple Design System Tokens */
:root {
  /* Primary */
  --color-apple-black: #000000;
  --color-apple-light: #f5f5f7;
  --color-apple-near-black: #1d1d1f;
  --color-apple-white: #ffffff;

  /* Interactive */
  --color-apple-blue: #0071e3;
  --color-apple-blue-hover: #0077ed;
  --color-apple-link: #0066cc;
  --color-apple-link-dark: #2997ff;

  /* Surface (dark sections) */
  --color-apple-card-dark: #1d1d1f;
  --color-apple-surface-dark: #272729;

  /* Functional */
  --color-apple-success: #22c55e;
  --color-apple-warning: #f59e0b;
  --color-apple-error: #ef4444;

  /* Shadow — inline styles에서 var(--shadow-apple)로 참조 */
  --shadow-apple: 3px 5px 30px 0px rgba(0, 0, 0, 0.22);
}

@theme inline {
  /* Colors — Tailwind utility classes (bg-apple-blue 등)로 사용 가능 */
  --color-apple-black: var(--color-apple-black);
  --color-apple-light: var(--color-apple-light);
  --color-apple-near-black: var(--color-apple-near-black);
  --color-apple-white: var(--color-apple-white);
  --color-apple-blue: var(--color-apple-blue);
  --color-apple-blue-hover: var(--color-apple-blue-hover);
  --color-apple-link: var(--color-apple-link);
  --color-apple-link-dark: var(--color-apple-link-dark);
  --color-apple-card-dark: var(--color-apple-card-dark);
  --color-apple-surface-dark: var(--color-apple-surface-dark);
  --color-apple-success: var(--color-apple-success);
  --color-apple-warning: var(--color-apple-warning);
  --color-apple-error: var(--color-apple-error);

  /* Font — System Font Stack */
  --font-sans: system-ui, -apple-system, BlinkMacSystemFont, 'SF Pro Display', 'SF Pro Text', 'Helvetica Neue', Arial, sans-serif;

  /* Shadow */
  --shadow-apple: var(--shadow-apple);
}

body {
  font-family: var(--font-sans);
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}
```

- [ ] **Step 2: 빌드 확인**

Run: `cd dashboard && pnpm build 2>&1 | tail -5`
Expected: 빌드 성공

- [ ] **Step 3: Commit**

```bash
git add dashboard/src/app/globals.css
git commit -m "style: Apple 디자인 토큰 정의 — Tailwind v4 @theme inline"
```

---

## Task 2: GlassNav 컴포넌트 + Layout 업데이트

**Files:**
- Create: `dashboard/src/components/GlassNav.tsx`
- Modify: `dashboard/src/app/layout.tsx`

공유 Glass Navigation 생성. 기존 각 페이지의 인라인 `<nav>`를 대체.

- [ ] **Step 1: GlassNav 컴포넌트 생성**

```tsx
// dashboard/src/components/GlassNav.tsx
'use client';

import { useState } from 'react';
import Link from 'next/link';
import { usePathname } from 'next/navigation';

const NAV_LINKS = [
  { href: '/', label: 'Home' },
  { href: '/history', label: 'History' },
  { href: '/settings', label: 'Settings' },
];

export function GlassNav() {
  const pathname = usePathname();
  const [menuOpen, setMenuOpen] = useState(false);

  return (
    <nav
      className="sticky top-0 z-50 h-12 flex items-center justify-between px-6 max-w-[980px] mx-auto"
      style={{
        background: 'rgba(0, 0, 0, 0.8)',
        backdropFilter: 'saturate(180%) blur(20px)',
        WebkitBackdropFilter: 'saturate(180%) blur(20px)',
      }}
      aria-label="Main navigation"
    >
      <Link href="/" className="text-white font-semibold" style={{ fontSize: '17px' }}>
        PushOver
      </Link>

      {/* Desktop links */}
      <div className="hidden sm:flex gap-5">
        {NAV_LINKS.map((link) => (
          <Link
            key={link.href}
            href={link.href}
            className="text-white/80 hover:text-white hover:underline"
            style={{ fontSize: '12px' }}
          >
            {link.label}
          </Link>
        ))}
      </div>

      {/* Mobile hamburger */}
      <button
        className="sm:hidden text-white text-lg"
        onClick={() => setMenuOpen(!menuOpen)}
        aria-label="Toggle menu"
      >
        {menuOpen ? '\u2715' : '\u2630'}
      </button>

      {/* Mobile overlay menu */}
      {menuOpen && (
        <div
          className="fixed inset-0 top-12 z-40 flex flex-col items-center gap-6 pt-12"
          style={{ background: 'rgba(0, 0, 0, 0.95)' }}
        >
          {NAV_LINKS.map((link) => (
            <Link
              key={link.href}
              href={link.href}
              onClick={() => setMenuOpen(false)}
              className="text-white text-xl font-medium"
            >
              {link.label}
            </Link>
          ))}
        </div>
      )}
    </nav>
  );
}
```

- [ ] **Step 2: layout.tsx 업데이트 — Geist 제거, GlassNav 포함**

```tsx
// dashboard/src/app/layout.tsx
import type { Metadata } from 'next';
import { GlassNav } from '@/components/GlassNav';
import './globals.css';

export const metadata: Metadata = {
  title: 'PushOver Dashboard',
  description: 'PushOver Serverless Platform',
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="ko" className="antialiased">
      <body className="min-h-full flex flex-col">
        <GlassNav />
        <main className="pt-12">{children}</main>
      </body>
    </html>
  );
}
```

- [ ] **Step 3: 빌드 확인**

Run: `cd dashboard && pnpm build 2>&1 | tail -5`
Expected: 빌드 성공 (기존 페이지에 미사용 Geist import 경고 무시)

- [ ] **Step 4: Commit**

```bash
git add dashboard/src/components/GlassNav.tsx dashboard/src/app/layout.tsx
git commit -m "feat: GlassNav 공유 컴포넌트 생성 및 layout.tsx 통합"
```

---

## Task 3: Home 페이지 리디자인

**Files:**
- Modify: `dashboard/src/app/page.tsx`

Black Hero 섹션(제목 + CTA) → Light 섹션(Stats 카드). 모달 Apple 스타일. 기존 비즈니스 로직(상태, API 호출, 이미지 처리) 유지.

- [ ] **Step 1: page.tsx 전체 교체**

```tsx
// dashboard/src/app/page.tsx
'use client';

import { useState, useEffect } from 'react';
import Link from 'next/link';
import { useSettings } from './settings/hooks/useSettings';
import { pushOverAPI } from '@/lib/api';

export default function Home() {
  const [showModal, setShowModal] = useState(false);
  const [message, setMessage] = useState('');
  const [title, setTitle] = useState('');
  const [sending, setSending] = useState(false);
  const [showBanner, setShowBanner] = useState(false);
  const [imageBase64, setImageBase64] = useState<string | null>(null);
  const [imagePreview, setImagePreview] = useState<string | null>(null);
  const { settings, isLoading, hasRequiredSettings } = useSettings();

  useEffect(() => {
    setShowBanner(!hasRequiredSettings);
  }, [hasRequiredSettings]);

  const handleSend = async () => {
    setSending(true);
    try {
      await pushOverAPI.sendMessage({ message, title: title || undefined, image: imageBase64 || undefined });
      alert('메시지 전송 성공!');
      setShowModal(false);
      setMessage('');
      setTitle('');
      setImageBase64(null);
      setImagePreview(null);
    } catch (error) {
      alert('전송 실패: ' + (error instanceof Error ? error.message : error));
    } finally {
      setSending(false);
    }
  };

  const handleImageChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    const reader = new FileReader();
    reader.onload = (ev) => {
      setImagePreview(ev.target?.result as string);
    };
    reader.readAsDataURL(file);

    const b64Reader = new FileReader();
    b64Reader.onload = (ev) => {
      const result = ev.target?.result as string;
      const base64 = result.split(',')[1];
      setImageBase64(base64);
    };
    b64Reader.readAsDataURL(file);
  };

  const clearImage = () => {
    setImageBase64(null);
    setImagePreview(null);
  };

  const closeModal = () => {
    setShowModal(false);
    setMessage('');
    setTitle('');
    clearImage();
  };

  // Modal scroll lock (Spec 7: 접근성 — 모달 오픈 시 overflow: hidden)
  useEffect(() => {
    if (showModal) {
      document.body.style.overflow = 'hidden';
    } else {
      document.body.style.overflow = '';
    }
    return () => { document.body.style.overflow = ''; };
  }, [showModal]);

  if (isLoading) {
    return (
      <div className="min-h-screen bg-black p-8">
        <div className="animate-pulse space-y-4 max-w-[980px] mx-auto">
          <div className="h-16 bg-white/10 rounded-2xl" />
          <div className="h-32 bg-white/10 rounded-2xl" />
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen">
      {/* Section 1: Black Hero */}
      <section className="bg-black text-white text-center py-20 px-6">
        <div className="max-w-[980px] mx-auto">
          <h1
            className="font-semibold mb-3"
            style={{ fontSize: 'clamp(28px, 5vw, 56px)', lineHeight: 1.07, letterSpacing: '-0.28px' }}
          >
            PushOver Serverless Platform
          </h1>
          <p className="text-white/60 mb-8" style={{ fontSize: '17px' }}>
            메시지 전송, 웹훅 관리, 기록 조회
          </p>

          {showBanner && (
            <div className="bg-amber-900/30 border border-amber-700 rounded-2xl p-4 mb-6 max-w-md mx-auto text-left">
              <div className="flex items-center gap-3">
                <span className="text-xl">⚠️</span>
                <div className="flex-1">
                  <p className="font-medium text-amber-200 text-sm">PushOver 설정이 필요합니다</p>
                  <p className="text-xs text-white/50">Settings에서 API Token과 User Key를 설정하세요</p>
                </div>
                <Link
                  href="/settings"
                  className="px-4 py-2 bg-amber-600 text-white rounded-[980px] text-sm font-medium hover:bg-amber-700 transition-colors whitespace-nowrap"
                >
                  설정하기
                </Link>
              </div>
            </div>
          )}

          <button
            onClick={() => setShowModal(true)}
            className="px-8 py-3 bg-[var(--color-apple-blue)] text-white rounded-[980px] font-medium hover:bg-[var(--color-apple-blue-hover)] transition-colors"
            style={{ fontSize: '17px' }}
          >
            메시지 보내기
          </button>
        </div>
      </section>

      {/* Section 2: Light Content */}
      <section className="bg-[var(--color-apple-light)] py-16 px-6">
        <div className="max-w-[980px] mx-auto grid grid-cols-1 sm:grid-cols-2 gap-6">
          <div className="bg-white rounded-[16px] p-6" style={{ boxShadow: 'var(--shadow-apple)' }}>
            <p className="text-xs font-semibold uppercase tracking-wide text-zinc-500 mb-2">Platform</p>
            <p className="text-[21px] font-bold text-[var(--color-apple-near-black)]">Cloudflare Workers</p>
            <p className="text-sm text-zinc-500 mt-1">Serverless 메시지 전송</p>
          </div>
          <div className="bg-white rounded-[16px] p-6" style={{ boxShadow: 'var(--shadow-apple)' }}>
            <p className="text-xs font-semibold uppercase tracking-wide text-zinc-500 mb-2">Storage</p>
            <p className="text-[21px] font-bold text-[var(--color-apple-near-black)]">D1 + KV + R2</p>
            <p className="text-sm text-zinc-500 mt-1">메시지 기록, 설정, 이미지</p>
          </div>
        </div>
      </section>

      {/* Message Modal */}
      {showModal && (
        <div className="fixed inset-0 z-50 flex items-center justify-center" onClick={closeModal}>
          <div className="absolute inset-0 bg-black/50" />
          <div
            className="relative bg-white rounded-[16px] max-w-md w-full mx-4 p-6"
            style={{ boxShadow: 'var(--shadow-apple)' }}
            onClick={(e) => e.stopPropagation()}
          >
            <h3 className="text-lg font-semibold mb-4 text-[var(--color-apple-near-black)]">메시지 보내기</h3>
            <div className="space-y-4">
              <div>
                <label className="block text-xs font-semibold uppercase tracking-wide text-zinc-500 mb-1.5">
                  제목 (선택)
                </label>
                <input
                  type="text"
                  value={title}
                  onChange={(e) => setTitle(e.target.value)}
                  placeholder="메시지 제목"
                  className="w-full px-3 py-3 rounded-[12px] bg-[var(--color-apple-light)] border-0 text-[var(--color-apple-near-black)] focus:outline-none focus:ring-2 focus:ring-[var(--color-apple-blue)]"
                />
              </div>
              <div>
                <label className="block text-xs font-semibold uppercase tracking-wide text-zinc-500 mb-1.5">
                  메시지
                </label>
                <textarea
                  value={message}
                  onChange={(e) => setMessage(e.target.value)}
                  placeholder="전송할 메시지"
                  rows={4}
                  required
                  className="w-full px-3 py-3 rounded-[12px] bg-[var(--color-apple-light)] border-0 text-[var(--color-apple-near-black)] focus:outline-none focus:ring-2 focus:ring-[var(--color-apple-blue)]"
                />
              </div>
              <div>
                <label className="block text-xs font-semibold uppercase tracking-wide text-zinc-500 mb-1.5">
                  이미지 첨부 (선택)
                </label>
                {imagePreview ? (
                  <div className="relative">
                    <img src={imagePreview} alt="preview" className="max-h-40 rounded-[12px]" />
                    <button
                      type="button"
                      onClick={clearImage}
                      className="absolute top-1 right-1 w-6 h-6 bg-black/70 text-white rounded-full text-xs flex items-center justify-center hover:bg-[var(--color-apple-error)]"
                    >
                      ✕
                    </button>
                  </div>
                ) : (
                  <input
                    type="file"
                    accept="image/*"
                    onChange={handleImageChange}
                    className="w-full text-sm text-zinc-400 file:mr-4 file:py-2 file:px-4 file:rounded-[980px] file:border file:border-[var(--color-apple-blue)] file:text-sm file:font-medium file:bg-transparent file:text-[var(--color-apple-blue)] hover:file:bg-[var(--color-apple-light)]"
                  />
                )}
              </div>
              <div className="flex gap-3 justify-end pt-2">
                <button
                  onClick={closeModal}
                  className="px-5 py-2.5 border border-[var(--color-apple-blue)] rounded-[980px] font-medium text-[var(--color-apple-blue)] hover:bg-[var(--color-apple-light)] transition-colors"
                >
                  취소
                </button>
                <button
                  onClick={handleSend}
                  disabled={!message || sending}
                  className="px-5 py-2.5 bg-[var(--color-apple-blue)] text-white rounded-[980px] font-medium hover:bg-[var(--color-apple-blue-hover)] disabled:opacity-50 transition-colors"
                >
                  {sending ? '전송 중...' : '전송'}
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
```

- [ ] **Step 2: 빌드 + 로컬 확인**

Run: `cd dashboard && pnpm build 2>&1 | tail -5`
Expected: 빌드 성공. `pnpm loc` 실행 후 http://localhost:3000 에서 Apple 스타일 확인.

- [ ] **Step 3: Commit**

```bash
git add dashboard/src/app/page.tsx
git commit -m "style: Home 페이지 Apple 디자인 리디자인 — 교차 섹션 + 모달"
```

---

## Task 4: History 페이지 리디자인

**Files:**
- Modify: `dashboard/src/app/history/page.tsx`

Black Hero → Light Table. Apple 테이블 스타일 + 상태 뱃지 Pill.

- [ ] **Step 1: history/page.tsx 전체 교체**

```tsx
// dashboard/src/app/history/page.tsx
'use client';

import { useEffect, useState } from 'react';
import { pushOverAPI, type Message } from '@/lib/api';

// Tailwind opacity modifier(/10)는 CSS variable에 작동 불가 → rgba 직접 사용
const STATUS_STYLES: Record<string, { bg: string; text: string }> = {
  sent:      { bg: 'rgba(0, 113, 227, 0.1)',  text: '#0071e3' },
  delivered: { bg: 'rgba(34, 197, 94, 0.1)',   text: '#22c55e' },
  failed:    { bg: 'rgba(239, 68, 68, 0.1)',   text: '#ef4444' },
  queued:    { bg: 'rgba(245, 158, 11, 0.1)',  text: '#f59e0b' },
};

export default function HistoryPage() {
  const [messages, setMessages] = useState<Message[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    pushOverAPI.getHistory()
      .then(setMessages)
      .catch((err) => setError(err instanceof Error ? err.message : 'Failed to load messages'))
      .finally(() => setLoading(false));
  }, []);

  return (
    <div className="min-h-screen">
      {/* Section 1: Black Hero */}
      <section className="bg-black text-white text-center py-16 px-6">
        <div className="max-w-[980px] mx-auto">
          <h1
            className="font-semibold mb-3"
            style={{ fontSize: 'clamp(24px, 4vw, 40px)', lineHeight: 1.10 }}
          >
            Message History
          </h1>
          <p className="text-white/60" style={{ fontSize: '17px' }}>전송된 메시지 기록</p>
        </div>
      </section>

      {/* Section 2: Light Table */}
      <section className="bg-[var(--color-apple-light)] py-12 px-6">
        <div className="max-w-[980px] mx-auto">
          <div className="bg-white rounded-[16px] overflow-hidden" style={{ boxShadow: 'var(--shadow-apple)' }}>
            {loading ? (
              <div className="px-6 py-12 text-center text-zinc-400">Loading...</div>
            ) : error ? (
              <div className="px-6 py-12 text-center text-[var(--color-apple-error)]">에러: {error}</div>
            ) : messages.length === 0 ? (
              <div className="px-6 py-12 text-center text-zinc-400">No messages found</div>
            ) : (
              <table className="w-full">
                <thead>
                  <tr className="border-b border-zinc-100">
                    <th className="px-6 py-3 text-left text-xs font-semibold uppercase tracking-wide text-zinc-500">Time</th>
                    <th className="px-6 py-3 text-left text-xs font-semibold uppercase tracking-wide text-zinc-500">Title</th>
                    <th className="px-6 py-3 text-left text-xs font-semibold uppercase tracking-wide text-zinc-500">Message</th>
                    <th className="px-6 py-3 text-left text-xs font-semibold uppercase tracking-wide text-zinc-500">Status</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-zinc-50">
                  {messages.map((msg) => (
                    <tr key={msg.id} className="hover:bg-zinc-50 transition-colors">
                      <td className="px-6 py-4 text-sm text-[var(--color-apple-near-black)]">
                        {msg.sent_at ? new Date(msg.sent_at + 'Z').toLocaleString() : new Date(msg.created_at + 'Z').toLocaleString()}
                      </td>
                      <td className="px-6 py-4 text-sm font-medium text-[var(--color-apple-near-black)]">
                        {msg.title || '-'}
                      </td>
                      <td className="px-6 py-4 text-sm text-zinc-600 max-w-md truncate">
                        {msg.message}
                      </td>
                      <td className="px-6 py-4">
                        <span
                          className="inline-flex items-center px-2.5 py-0.5 rounded-[980px] text-xs font-medium"
                          style={{
                            backgroundColor: STATUS_STYLES[msg.status]?.bg || 'rgba(0,0,0,0.05)',
                            color: STATUS_STYLES[msg.status]?.text || '#52525b',
                          }}
                        >
                          {msg.status}
                        </span>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )}
          </div>
        </div>
      </section>
    </div>
  );
}
```

- [ ] **Step 2: 빌드 확인**

Run: `cd dashboard && pnpm build 2>&1 | tail -5`
Expected: 빌드 성공

- [ ] **Step 3: Commit**

```bash
git add dashboard/src/app/history/page.tsx
git commit -m "style: History 페이지 Apple 디자인 리디자인 — Black Hero + Light Table"
```

---

## Task 5: Settings 탭 컴포넌트 리디자인

**Files:**
- Modify: `dashboard/src/app/settings/components/PushOverTab.tsx`
- Modify: `dashboard/src/app/settings/components/WorkerTab.tsx`
- Modify: `dashboard/src/app/settings/components/NotificationTab.tsx`

3개 탭 컴포넌트에 Apple 폼 필드 스타일 적용 (`bg-f5f5f7`, `rounded-xl`, `border-0`) + Pill 버튼.

- [ ] **Step 1: PushOverTab.tsx 교체**

```tsx
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

    if (!settings.worker.token) {
      setTestResult({ success: false, message: 'Worker 설정에서 Worker Token을 입력해주세요' });
      return;
    }

    setIsTesting(true);
    setTestResult(null);

    try {
      const response = await fetch(`${settings.worker.url}/api/v1/messages`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${settings.worker.token}`,
        },
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

  const inputClass = 'w-full px-3 py-3 rounded-[12px] bg-[var(--color-apple-light)] border-0 text-[var(--color-apple-near-black)] font-mono text-sm focus:outline-none focus:ring-2 focus:ring-[var(--color-apple-blue)]';

  return (
    <div className="space-y-5">
      <div>
        <label className="block text-xs font-semibold uppercase tracking-wide text-zinc-500 mb-1.5">
          API Token <span className="text-[var(--color-apple-error)]">*</span>
        </label>
        <input
          type="password"
          value={apiToken}
          onChange={(e) => setApiToken(e.target.value)}
          placeholder="azGDORePK8gMaC0QOYAMyEL..."
          className={inputClass}
        />
        <p className="text-xs text-zinc-400 mt-1">pushover.net/apps 에서 확인</p>
      </div>

      <div>
        <label className="block text-xs font-semibold uppercase tracking-wide text-zinc-500 mb-1.5">
          User Key <span className="text-[var(--color-apple-error)]">*</span>
        </label>
        <input
          type="password"
          value={userKey}
          onChange={(e) => setUserKey(e.target.value)}
          placeholder="uQiRzpo4DXghDmr9QzzfQu27cmVRsG..."
          className={inputClass}
        />
        <p className="text-xs text-zinc-400 mt-1">pushover.net 메인 페이지 상단</p>
      </div>

      {testResult && (
        <div className={`p-3 rounded-[12px] ${testResult.success ? 'bg-green-50 border border-green-200 text-green-700' : 'bg-red-50 border border-red-200 text-red-700'}`}>
          <p className="text-sm">{testResult.message}</p>
        </div>
      )}

      <div className="flex gap-3 pt-2">
        <button
          onClick={handleSave}
          className="px-5 py-2.5 bg-[var(--color-apple-blue)] text-white rounded-[980px] font-medium hover:bg-[var(--color-apple-blue-hover)] transition-colors"
        >
          저장
        </button>
        <button
          onClick={handleTest}
          disabled={isTesting}
          className="px-5 py-2.5 border border-[var(--color-apple-blue)] text-[var(--color-apple-blue)] rounded-[980px] font-medium hover:bg-[var(--color-apple-light)] transition-colors disabled:opacity-50"
        >
          {isTesting ? '테스트 중...' : '테스트'}
        </button>
      </div>
    </div>
  );
}
```

- [ ] **Step 2: WorkerTab.tsx 교체**

```tsx
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
  const [token, setToken] = useState(settings.worker.token || '');
  const [showResetConfirm, setShowResetConfirm] = useState(false);

  useEffect(() => {
    setUrl(settings.worker.url);
    setToken(settings.worker.token || '');
  }, [settings.worker]);

  const handleSave = () => {
    onUpdate({
      worker: { url, token: token || undefined }
    });
  };

  const handleReset = () => {
    if (showResetConfirm) {
      onReset();
      setShowResetConfirm(false);
    } else {
      setShowResetConfirm(true);
    }
  };

  const inputClass = 'w-full px-3 py-3 rounded-[12px] bg-[var(--color-apple-light)] border-0 text-[var(--color-apple-near-black)] font-mono text-sm focus:outline-none focus:ring-2 focus:ring-[var(--color-apple-blue)]';

  return (
    <div className="space-y-5">
      <div>
        <label className="block text-xs font-semibold uppercase tracking-wide text-zinc-500 mb-1.5">
          Worker URL <span className="text-[var(--color-apple-error)]">*</span>
        </label>
        <input
          type="text"
          value={url}
          onChange={(e) => setUrl(e.target.value)}
          placeholder="https://pushover-worker.cromksy.workers.dev"
          className={inputClass}
        />
        <p className="text-xs text-zinc-400 mt-1">Cloudflare Worker 배포 URL</p>
      </div>

      <div>
        <label className="block text-xs font-semibold uppercase tracking-wide text-zinc-500 mb-1.5">
          Worker Token (CF_WORKER_TOKEN) <span className="text-[var(--color-apple-error)]">*</span>
        </label>
        <input
          type="password"
          value={token}
          onChange={(e) => setToken(e.target.value)}
          placeholder="Worker API 인증 토큰"
          className={inputClass}
        />
        <p className="text-xs text-zinc-400 mt-1">Worker D1에 등록된 인증 토큰</p>
      </div>

      <div className="flex gap-3 pt-2">
        <button
          onClick={handleSave}
          className="px-5 py-2.5 bg-[var(--color-apple-blue)] text-white rounded-[980px] font-medium hover:bg-[var(--color-apple-blue-hover)] transition-colors"
        >
          저장
        </button>
        <button
          onClick={handleReset}
          className={`px-5 py-2.5 rounded-[980px] font-medium transition-colors ${
            showResetConfirm
              ? 'bg-[var(--color-apple-error)] text-white hover:opacity-90'
              : 'border border-[var(--color-apple-error)] text-[var(--color-apple-error)] hover:bg-red-50'
          }`}
        >
          {showResetConfirm ? '확인' : '초기화'}
        </button>
        {showResetConfirm && (
          <button
            onClick={() => setShowResetConfirm(false)}
            className="px-5 py-2.5 border border-zinc-300 text-zinc-600 rounded-[980px] font-medium hover:bg-[var(--color-apple-light)] transition-colors"
          >
            취소
          </button>
        )}
      </div>
    </div>
  );
}
```

- [ ] **Step 3: NotificationTab.tsx 교체**

```tsx
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
      setShowResetConfirm(false);
    } else {
      setShowResetConfirm(true);
    }
  };

  const selectClass = 'w-full px-3 py-3 rounded-[12px] bg-[var(--color-apple-light)] border-0 text-[var(--color-apple-near-black)] focus:outline-none focus:ring-2 focus:ring-[var(--color-apple-blue)] appearance-none';

  return (
    <div className="space-y-5">
      <div>
        <label className="block text-xs font-semibold uppercase tracking-wide text-zinc-500 mb-1.5">기본 사운드</label>
        <select value={sound} onChange={(e) => setSound(e.target.value)} className={selectClass}>
          {SOUND_OPTIONS.map((opt) => (
            <option key={opt} value={opt}>{opt === 'pushover' ? 'PushOver (기본)' : opt}</option>
          ))}
        </select>
        <p className="text-xs text-zinc-400 mt-1">기기에서 설정한 사운드가 우선됩니다</p>
      </div>

      <div>
        <label className="block text-xs font-semibold uppercase tracking-wide text-zinc-500 mb-1.5">기본 기기</label>
        <select value="all" disabled className={`${selectClass} opacity-60`}>
          <option value="all">모든 기기</option>
        </select>
        <p className="text-xs text-zinc-400 mt-1">v1에서는 모든 기기만 지원</p>
      </div>

      <div>
        <label className="block text-xs font-semibold uppercase tracking-wide text-zinc-500 mb-1.5">우선순위</label>
        <select value={priority} onChange={(e) => setPriority(Number(e.target.value))} className={selectClass}>
          {PRIORITY_OPTIONS.map((opt) => (
            <option key={opt.value} value={opt.value}>{opt.label}</option>
          ))}
        </select>
      </div>

      <div className="flex gap-3 pt-2">
        <button
          onClick={handleSave}
          className="px-5 py-2.5 bg-[var(--color-apple-blue)] text-white rounded-[980px] font-medium hover:bg-[var(--color-apple-blue-hover)] transition-colors"
        >
          저장
        </button>
        <button
          onClick={handleReset}
          className={`px-5 py-2.5 rounded-[980px] font-medium transition-colors ${
            showResetConfirm
              ? 'bg-[var(--color-apple-error)] text-white hover:opacity-90'
              : 'border border-[var(--color-apple-error)] text-[var(--color-apple-error)] hover:bg-red-50'
          }`}
        >
          {showResetConfirm ? '확인' : '초기화'}
        </button>
        {showResetConfirm && (
          <button
            onClick={() => setShowResetConfirm(false)}
            className="px-5 py-2.5 border border-zinc-300 text-zinc-600 rounded-[980px] font-medium hover:bg-[var(--color-apple-light)] transition-colors"
          >
            취소
          </button>
        )}
      </div>
    </div>
  );
}
```

- [ ] **Step 4: 빌드 확인**

Run: `cd dashboard && pnpm build 2>&1 | tail -5`
Expected: 빌드 성공

- [ ] **Step 5: Commit**

```bash
git add dashboard/src/app/settings/components/
git commit -m "style: Settings 탭 컴포넌트 Apple 폼 + Pill 버튼 적용"
```

---

## Task 6: Settings 페이지 리디자인

**Files:**
- Modify: `dashboard/src/app/settings/page.tsx`

Black Hero → Light Form. Apple 탭 바 (Pill active/inactive). 기존 탭 로직 유지.

- [ ] **Step 1: settings/page.tsx 전체 교체**

```tsx
// dashboard/src/app/settings/page.tsx
'use client';

import { useState } from 'react';
import { useSettings } from './hooks/useSettings';
import { PushOverTab } from './components/PushOverTab';
import { WorkerTab } from './components/WorkerTab';
import { NotificationTab } from './components/NotificationTab';

type TabType = 'pushover' | 'worker' | 'notification';

const TABS: { id: TabType; label: string }[] = [
  { id: 'pushover', label: 'PushOver' },
  { id: 'worker', label: 'Worker' },
  { id: 'notification', label: '알림' },
];

export default function SettingsPage() {
  const [activeTab, setActiveTab] = useState<TabType>('pushover');
  const { settings, isLoading, error, updateSettings, resetTab } = useSettings();

  if (isLoading) {
    return (
      <div className="min-h-screen bg-black p-8">
        <div className="animate-pulse space-y-4 max-w-[680px] mx-auto">
          <div className="h-10 bg-white/10 rounded-2xl" />
          <div className="h-10 bg-white/10 rounded-2xl" />
          <div className="h-10 bg-white/10 rounded-2xl w-1/2" />
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen">
      {/* Section 1: Black Hero */}
      <section className="bg-black text-white text-center py-16 px-6">
        <div className="max-w-[980px] mx-auto">
          <h1
            className="font-semibold mb-3"
            style={{ fontSize: 'clamp(24px, 4vw, 40px)', lineHeight: 1.10 }}
          >
            Settings
          </h1>
          <p className="text-white/60" style={{ fontSize: '17px' }}>PushOver 설정 관리</p>
        </div>
      </section>

      {/* Section 2: Light Form */}
      <section className="bg-[var(--color-apple-light)] py-10 px-6">
        <div className="max-w-[680px] mx-auto">
          {/* Tab Bar */}
          <div className="flex gap-1 mb-6 p-1 bg-white rounded-[12px]" style={{ boxShadow: 'var(--shadow-apple)' }}>
            {TABS.map((tab) => (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id)}
                className={`flex-1 px-4 py-2.5 rounded-[980px] text-sm font-medium transition-colors ${
                  activeTab === tab.id
                    ? 'bg-[var(--color-apple-blue)] text-white'
                    : 'text-zinc-500 hover:text-[var(--color-apple-near-black)]'
                }`}
              >
                {tab.label}
              </button>
            ))}
          </div>

          {error && (
            <div className="mb-4 p-3 bg-red-50 border border-red-200 rounded-[12px]" role="alert">
              <p className="text-sm text-red-700">{error}</p>
            </div>
          )}

          {/* Form Card */}
          <div className="bg-white rounded-[16px] p-6" style={{ boxShadow: 'var(--shadow-apple)' }}>
            {activeTab === 'pushover' && <PushOverTab settings={settings} onUpdate={updateSettings} />}
            {activeTab === 'worker' && <WorkerTab settings={settings} onUpdate={updateSettings} onReset={() => resetTab('worker')} />}
            {activeTab === 'notification' && <NotificationTab settings={settings} onUpdate={updateSettings} onReset={() => resetTab('notification')} />}
          </div>
        </div>
      </section>
    </div>
  );
}
```

- [ ] **Step 2: 빌드 확인**

Run: `cd dashboard && pnpm build 2>&1 | tail -5`
Expected: 빌드 성공

- [ ] **Step 3: Commit**

```bash
git add dashboard/src/app/settings/page.tsx
git commit -m "style: Settings 페이지 Apple 디자인 리디자인 — Black Hero + Tab Form"
```

---

## Task 7: E2E 테스트 업데이트

**Files:**
- Modify: `dashboard/tests/loc.spec.ts`

새 마크업에 맞게 선택자(selector) 업데이트. 테스트 로직(검증, mock)은 동일.

주요 변경:
- `page.locator('h1')` → 여전히 h1 사용하므로 대부분 호환
- `textarea[placeholder="전송할 메시지"]` → 유지
- `page.getByRole('button', { name: '저장' })` → 유지
- `page.locator('img[alt="preview"]')` → 유지
- `page.getByRole('button', { name: '전송 중...' })` → 유지

- [ ] **Step 1: loc.spec.ts 선택자 검증**

테스트를 실행하여 기존 선택자가 새 마크업에서 동작하는지 확인.

Run: `cd dashboard && pnpm test:loc 2>&1 | tail -20`
Expected: 4/4 테스트 통과

실패하는 선택자가 있으면 아래와 같이 수정:

```typescript
// Home 페이지 h1 텍스트 변경 반영 (필요 시)
// 기존: 'PushOver Dashboard' → 신규: 'PushOver Serverless Platform'
// loc.spec.ts line 123:
await expect(page.locator('h1')).toContainText('PushOver Serverless Platform');

// loc.spec.ts line 157:
await expect(page.locator('h1')).toContainText('PushOver Serverless Platform');
```

- [ ] **Step 2: 전체 E2E 테스트 실행**

Run: `cd dashboard && pnpm test:loc 2>&1 | tail -20`
Expected: 4/4 통과

- [ ] **Step 3: Commit (변경 있는 경우만)**

```bash
git add dashboard/tests/loc.spec.ts
git commit -m "test: E2E 선택자 Apple 리디자인 마크업에 맞게 업데이트"
```

---

## Task 8: 최종 검증 + 정리

- [ ] **Step 1: 전체 빌드**

Run: `cd dashboard && pnpm build 2>&1 | tail -10`
Expected: 빌드 성공

- [ ] **Step 2: E2E 테스트 전체 실행**

Run: `cd dashboard && pnpm test:loc 2>&1 | tail -20`
Expected: 4/4 통과

- [ ] **Step 3: 미사용 파일 정리 확인**

`public/` 아래 기본 Next.js 파일(file.svg, globe.svg, next.svg, vercel.svg, window.svg)이 여전히 존재하는지 확인. 삭제하지 않음 (기본 에셋).

- [ ] **Step 4: 로컬 시각 확인**

Run: `cd dashboard && pnpm loc`
http://localhost:3000 에서 다음 확인:
1. Glass Nav 표시 (sticky, 반투명)
2. Home: Black hero + Light stats + Apple 모달
3. History (`/history`): Black hero + Light table
4. Settings (`/settings`): Black hero + Tab bar + Light form
5. 모바일 반응형 (브라우저 width 375px)

- [ ] **Step 5: 최종 Commit (미커밋 변경 있는 경우)**

```bash
git add -A
git commit -m "style: Apple Design System Dashboard 리디자인 완료"
```
