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
