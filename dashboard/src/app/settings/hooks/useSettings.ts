'use client';

import { useState, useEffect, useCallback } from 'react';
import {
  Settings,
  DEFAULT_VALUES,
  loadSettings,
  saveSettings
} from '@/lib/settings';

interface UseSettingsReturn {
  settings: Settings;
  isLoading: boolean;
  error: string | null;
  updateSettings: (updates: Partial<Settings>) => void;
  resetTab: (tab: keyof Settings) => void;
  hasRequiredSettings: boolean;
}

export function useSettings(): UseSettingsReturn {
  const [settings, setSettings] = useState<Settings>(DEFAULT_VALUES);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const stored = loadSettings();
    if (stored) {
      setSettings(stored);
    }
    setIsLoading(false);
  }, []);

  const updateSettings = useCallback((updates: Partial<Settings>) => {
    setSettings(prev => {
      const merged = { ...prev };
      if (updates.pushover) {
        merged.pushover = { ...prev.pushover, ...updates.pushover };
      }
      if (updates.worker) {
        merged.worker = { ...prev.worker, ...updates.worker };
      }
      if (updates.notification) {
        merged.notification = { ...prev.notification, ...updates.notification };
      }
      try {
        saveSettings(merged);
        setError(null);
      } catch (e) {
        setError('설정 저장에 실패했습니다');
      }
      return merged;
    });
  }, []);

  const resetTab = useCallback((tab: keyof Settings) => {
    updateSettings({ [tab]: DEFAULT_VALUES[tab] });
  }, [updateSettings]);

  const hasRequiredSettings = Boolean(
    settings.pushover.apiToken && settings.pushover.userKey
  );

  return { settings, isLoading, error, updateSettings, resetTab, hasRequiredSettings };
}
