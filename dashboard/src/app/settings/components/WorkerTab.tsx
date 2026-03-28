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
      // useEffect가 settings.worker 변경을 자동으로 반영함
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
