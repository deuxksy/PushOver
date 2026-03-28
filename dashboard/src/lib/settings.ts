// dashboard/src/lib/settings.ts

export interface Settings {
  pushover: {
    apiToken: string;
    userKey: string;
  };
  worker: {
    url: string;
    token?: string;
    webhookSecret?: string;
  };
  notification: {
    sound: string;
    device: string;
    priority: number;
  };
}

export const DEFAULT_VALUES: Settings = {
  pushover: { apiToken: '', userKey: '' },
  worker: { url: 'https://pushover-worker.cromksy.workers.dev' },
  notification: { sound: 'pushover', device: 'all', priority: 0 }
};

export const SOUND_OPTIONS = [
  'pushover', 'bike', 'bugle', 'cashregister', 'classical', 'cosmic',
  'falling', 'gamelan', 'incoming', 'intermission', 'magic', 'mechanical',
  'pianobar', 'siren', 'spacealarm', 'tugboat', 'alien', 'climb',
  'persistent', 'echo', 'updown', 'vibrate', 'none'
] as const;

export const PRIORITY_OPTIONS = [
  { value: -2, label: '최저 (방해 금지 시 무음)' },
  { value: -1, label: '낮음 (소리 없이 배지만)' },
  { value: 0, label: '보통 (기본)' },
  { value: 1, label: '높음 (방해 금지 무시)' },
  { value: 2, label: '긴급 (확인 시까지 반복)' }
] as const;

export const SETTINGS_STORAGE_KEY = 'pushover-settings';

export function loadSettings(): Settings | null {
  if (typeof window === 'undefined') return null;
  try {
    const stored = localStorage.getItem(SETTINGS_STORAGE_KEY);
    if (stored) {
      return JSON.parse(atob(stored));
    }
  } catch (e) {
    console.error('Settings load failed:', {
      error: e instanceof Error ? e.message : String(e),
      key: SETTINGS_STORAGE_KEY
    });
  }
  return null;
}

export function saveSettings(settings: Settings): void {
  try {
    localStorage.setItem(SETTINGS_STORAGE_KEY, btoa(JSON.stringify(settings)));
  } catch (e) {
    throw new Error(
      `Settings save failed: ${e instanceof Error ? e.message : String(e)}`
    );
  }
}

export function validateSettings(settings: Partial<Settings>): string[] {
  const errors: string[] = [];

  if (settings.pushover?.apiToken && !/^[a-zA-Z0-9]{30}$/.test(settings.pushover.apiToken)) {
    errors.push('API Token 형식이 올바르지 않습니다');
  }
  if (settings.pushover?.userKey && !/^[a-zA-Z0-9]{30}$/.test(settings.pushover.userKey)) {
    errors.push('User Key 형식이 올바르지 않습니다');
  }
  if (settings.worker?.url) {
    try {
      new URL(settings.worker.url);
    } catch {
      errors.push('Worker URL 형식이 올바르지 않습니다');
    }
  }

  if (settings.notification?.sound && !SOUND_OPTIONS.includes(settings.notification.sound as any)) {
    errors.push('유효하지 않은 사운드입니다');
  }

  if (settings.notification?.priority !== undefined && (settings.notification.priority < -2 || settings.notification.priority > 2)) {
    errors.push('우선순위는 -2에서 2 사이여야 합니다');
  }

  return errors;
}
