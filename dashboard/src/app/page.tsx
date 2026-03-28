'use client';

import { useState, useEffect } from 'react';
import Link from 'next/link';
import { useSettings } from './settings/hooks/useSettings';

export default function Home() {
  const [showModal, setShowModal] = useState(false);
  const [message, setMessage] = useState('');
  const [title, setTitle] = useState('');
  const [sending, setSending] = useState(false);
  const [showBanner, setShowBanner] = useState(false);
  const { settings, isLoading, hasRequiredSettings } = useSettings();

  useEffect(() => {
    setShowBanner(!hasRequiredSettings);
  }, [hasRequiredSettings]);

  const handleSend = async () => {
    setSending(true);
    try {
      // TODO: Call API
      await new Promise(resolve => setTimeout(resolve, 1000));
      alert('메시지 전송 성공!');
      setShowModal(false);
      setMessage('');
      setTitle('');
    } catch (error) {
      alert('전송 실패: ' + error);
    } finally {
      setSending(false);
    }
  };

  if (isLoading) {
    return (
      <div className="min-h-screen bg-zinc-950 p-8">
        <div className="animate-pulse space-y-4 max-w-7xl mx-auto">
          <div className="h-16 bg-zinc-800 rounded" />
          <div className="h-32 bg-zinc-800 rounded" />
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-zinc-950">
      <nav className="border-b border-zinc-800 bg-black">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex h-16 items-center justify-between">
            <h1 className="text-xl font-bold text-zinc-100">PushOver Dashboard</h1>
            <div className="flex gap-4">
              <Link
                href="/history"
                className="px-4 py-2 text-sm font-medium text-zinc-300 hover:text-zinc-100"
              >
                History
              </Link>
              <Link
                href="/settings"
                className="px-4 py-2 text-sm font-medium text-zinc-300 hover:text-zinc-100"
              >
                Settings
              </Link>
            </div>
          </div>
        </div>
      </nav>

      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
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

        <div className="text-center py-20">
          <h2 className="text-3xl font-bold tracking-tight text-zinc-100 sm:text-4xl mb-4">
            PushOver Serverless Platform
          </h2>
          <p className="text-lg text-zinc-400 mb-8">
            메시지 전송, 웹훅 관리, 기록 조회
          </p>
          <button
            onClick={() => setShowModal(true)}
            className="px-6 py-3 bg-blue-600 text-white rounded-lg font-medium hover:bg-blue-700 transition-colors"
          >
            메시지 보내기
          </button>
        </div>
      </main>

      {showModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-zinc-900 rounded-lg shadow-xl max-w-md w-full mx-4 p-6">
            <h3 className="text-lg font-semibold mb-4 text-zinc-100">메시지 보내기</h3>
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-zinc-300 mb-2">
                  제목 (선택)
                </label>
                <input
                  type="text"
                  value={title}
                  onChange={(e) => setTitle(e.target.value)}
                  placeholder="메시지 제목"
                  className="w-full px-3 py-2 border border-zinc-700 rounded-lg bg-zinc-800 text-zinc-100"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-zinc-300 mb-2">
                  메시지
                </label>
                <textarea
                  value={message}
                  onChange={(e) => setMessage(e.target.value)}
                  placeholder="전송할 메시지"
                  rows={4}
                  required
                  className="w-full px-3 py-2 border border-zinc-700 rounded-lg bg-zinc-800 text-zinc-100"
                />
              </div>
              <div className="flex gap-3 justify-end">
                <button
                  onClick={() => {
                    setShowModal(false);
                    setMessage('');
                    setTitle('');
                  }}
                  className="px-4 py-2 border border-zinc-700 rounded-lg font-medium hover:bg-zinc-800 transition-colors text-zinc-300"
                >
                  취소
                </button>
                <button
                  onClick={handleSend}
                  disabled={!message || sending}
                  className="px-4 py-2 bg-blue-600 text-white rounded-lg font-medium hover:bg-blue-700 disabled:opacity-50 transition-colors"
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
