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
      // useEffect가 settings.notification 변경을 자동으로 반영함
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
