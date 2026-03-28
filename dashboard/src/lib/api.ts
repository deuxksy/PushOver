import { Settings, DEFAULT_VALUES, loadSettings } from './settings';

export interface Message {
  id: string;
  message: string;
  title?: string;
  status: string;
  sent_at: string;
  delivered_at?: string;
  acknowledged_at?: string;
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
  private settings: Settings;

  constructor() {
    const stored = loadSettings();
    this.settings = stored ?? {
      pushover: { ...DEFAULT_VALUES.pushover },
      worker: { ...DEFAULT_VALUES.worker },
      notification: { ...DEFAULT_VALUES.notification }
    };
  }

  private async request<T>(endpoint: string, options?: RequestInit): Promise<T> {
    const workerUrl = this.settings.worker?.url || DEFAULT_VALUES.worker.url;

    const response = await fetch(`${workerUrl}${endpoint}`, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        ...(this.settings.pushover?.apiToken && {
          'Authorization': `Bearer ${this.settings.pushover.apiToken}`
        }),
        ...(this.settings.worker?.webhookSecret && {
          'X-Webhook-Secret': this.settings.worker.webhookSecret
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
    return this.request<Message[]>(`/api/v1/messages?limit=${limit}`);
  }

  async sendMessage(data: SendMessageRequest): Promise<SendMessageResponse> {
    // 필수 설정 검증
    if (!this.settings.pushover?.apiToken || !this.settings.pushover?.userKey) {
      throw new Error('PushOver credentials not configured');
    }

    // PushOver API는 token/user를 body에 포함
    return this.request('/api/v1/messages', {
      method: 'POST',
      body: JSON.stringify({
        token: this.settings.pushover.apiToken,
        user: this.settings.pushover.userKey,
        message: data.message,
        title: data.title,
        sound: data.sound || this.settings.notification?.sound || DEFAULT_VALUES.notification.sound,
        device: data.device || this.settings.notification?.device || DEFAULT_VALUES.notification.device,
        priority: data.priority ?? this.settings.notification?.priority ?? DEFAULT_VALUES.notification.priority,
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
