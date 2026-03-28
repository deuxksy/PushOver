import { Settings, DEFAULT_VALUES, loadSettings } from './settings';

export interface Message {
  id: string;
  message: string;
  title?: string;
  status: string;
  sent_at: string | null;
  delivered_at?: string | null;
  acknowledged_at?: string | null;
  created_at: string;
}

export interface SendMessageRequest {
  message: string;
  title?: string;
  priority?: number;
  sound?: string;
  device?: string;
  url?: string;
  url_title?: string;
  html?: boolean;
}

export interface SendMessageResponse {
  status: number;
  request: string;
  receipt?: string;
}

export class PushOverAPI {
  private getSettings(): Settings {
    const stored = loadSettings();
    return stored ?? {
      pushover: { ...DEFAULT_VALUES.pushover },
      worker: { ...DEFAULT_VALUES.worker },
      notification: { ...DEFAULT_VALUES.notification }
    };
  }

  private async request<T>(endpoint: string, options?: RequestInit): Promise<T> {
    const settings = this.getSettings();
    const workerUrl = settings.worker?.url || DEFAULT_VALUES.worker.url;

    const response = await fetch(`${workerUrl}${endpoint}`, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        ...(settings.pushover?.apiToken && {
          'Authorization': `Bearer ${settings.pushover.apiToken}`
        }),
        ...(settings.worker?.webhookSecret && {
          'X-Webhook-Secret': settings.worker.webhookSecret
        }),
        ...options?.headers,
      },
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({ message: 'Unknown error' }));
      throw new Error(error.message || `API Error: ${response.status}`);
    }

    return response.json();
  }

  async getHistory(limit = 50): Promise<Message[]> {
    const data = await this.request<{ status: string; messages: Message[] }>(`/api/v1/messages?limit=${limit}`);
    return data.messages;
  }

  async sendMessage(data: SendMessageRequest): Promise<SendMessageResponse> {
    const settings = this.getSettings();

    // 필수 설정 검증
    if (!settings.pushover?.apiToken || !settings.pushover?.userKey) {
      throw new Error('PushOver credentials not configured');
    }

    // PushOver API는 token/user를 body에 포함
    return this.request('/api/v1/messages', {
      method: 'POST',
      body: JSON.stringify({
        token: settings.pushover.apiToken,
        user: settings.pushover.userKey,
        message: data.message,
        title: data.title,
        sound: data.sound || settings.notification?.sound || DEFAULT_VALUES.notification.sound,
        device: data.device || settings.notification?.device || DEFAULT_VALUES.notification.device,
        priority: data.priority ?? settings.notification?.priority ?? DEFAULT_VALUES.notification.priority,
        url: data.url,
        url_title: data.url_title,
        html: data.html,
      }),
    });
  }

  async getStatus(receipt: string): Promise<{ status: string; acknowledged: boolean }> {
    return this.request(`/api/v1/messages/${receipt}/status`);
  }
}

export const pushOverAPI = new PushOverAPI();
