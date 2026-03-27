'use client';

import { useState } from 'react';
import Link from 'next/link';
import { useSettings } from './hooks/useSettings';
import { PushOverTab } from './components/PushOverTab';
import { WorkerTab } from './components/WorkerTab';
import { NotificationTab } from './components/NotificationTab';

type TabType = 'pushover' | 'worker' | 'notification';

const TABS: { id: TabType; label: string; color: string }[] = [
  { id: 'pushover', label: 'PushOver', color: 'blue' },
  { id: 'worker', label: 'Worker', color: 'green' },
  { id: 'notification', label: '알림', color: 'amber' }
];

export default function SettingsPage() {
  const [activeTab, setActiveTab] = useState<TabType>('pushover');
  const { settings, isLoading, error, updateSettings, resetTab } = useSettings();

  if (isLoading) {
    return (
      <div className="min-h-screen bg-zinc-950 p-8">
        <div className="animate-pulse space-y-4 max-w-2xl mx-auto">
          <div className="h-10 bg-zinc-800 rounded" />
          <div className="h-10 bg-zinc-800 rounded" />
          <div className="h-10 bg-zinc-800 rounded w-1/2" />
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-zinc-950">
      <nav className="border-b border-zinc-800 bg-zinc-950">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex h-16 items-center justify-between">
            <h1 className="text-xl font-bold text-zinc-100">Settings</h1>
            <Link href="/" className="px-4 py-2 text-sm font-medium text-zinc-400 hover:text-zinc-100">
              ← Home
            </Link>
          </div>
        </div>
      </nav>

      <main className="max-w-2xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <div className="flex gap-1 border-b border-zinc-800 mb-6">
          {TABS.map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`px-4 py-3 text-sm font-medium rounded-t-lg transition-colors ${
                activeTab === tab.id
                  ? tab.color === 'blue' ? 'bg-blue-600 text-white'
                    : tab.color === 'green' ? 'bg-green-600 text-white'
                    : 'bg-amber-600 text-white'
                  : 'text-zinc-400 hover:text-zinc-100'
              }`}
            >
              {tab.label}
            </button>
          ))}
        </div>

        {error && (
          <div className="mb-4 p-3 bg-red-900/30 border border-red-700 rounded-lg">
            <p className="text-sm text-red-400">{error}</p>
          </div>
        )}

        <div className="bg-zinc-900 border border-zinc-800 rounded-lg p-6">
          {activeTab === 'pushover' && <PushOverTab settings={settings} onUpdate={updateSettings} />}
          {activeTab === 'worker' && <WorkerTab settings={settings} onUpdate={updateSettings} onReset={() => resetTab('worker')} />}
          {activeTab === 'notification' && <NotificationTab settings={settings} onUpdate={updateSettings} onReset={() => resetTab('notification')} />}
        </div>
      </main>
    </div>
  );
}
