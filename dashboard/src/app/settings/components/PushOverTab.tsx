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
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${apiToken}`,
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
