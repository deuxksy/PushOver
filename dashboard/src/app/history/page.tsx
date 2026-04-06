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
