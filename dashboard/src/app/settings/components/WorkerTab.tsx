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
