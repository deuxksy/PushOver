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
