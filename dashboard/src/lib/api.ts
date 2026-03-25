const API_BASE = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8787';

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
  url?: string;
  url_title?: string;
  html?: boolean;
}

export class PushOverAPI {
  private apiKey: string;

  constructor(apiKey: string) {
    this.apiKey = apiKey;
  }

  private async request<T>(
    endpoint: string,
    options?: RequestInit
  ): Promise<T> {
    const response = await fetch(`${API_BASE}${endpoint}`, {
      ...options,
      headers: {
        'Authorization': `Bearer ${this.apiKey}`,
        'Content-Type': 'application/json',
        ...options?.headers,
      },
    });

    if (!response.ok) {
      throw new Error(`API Error: ${response.status}`);
    }

    return response.json();
  }

  async getHistory(limit = 50): Promise<Message[]> {
    return this.request<Message[]>('/api/v1/messages');
  }

  async sendMessage(data: SendMessageRequest): Promise<{ status: string; request: string }> {
    return this.request('/api/v1/messages', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  async getStatus(receipt: string): Promise<{ status: string; acknowledged: boolean }> {
    return this.request(`/api/v1/messages/${receipt}/status`);
  }
}
