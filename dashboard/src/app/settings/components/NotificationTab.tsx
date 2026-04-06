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
