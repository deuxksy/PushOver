import { test, expect } from '@playwright/test';
import path from 'path';

const MOCK_WORKER_URL = 'https://pushover-worker.cromksy.workers.dev';

// mock API 응답
const MOCK_SEND_RESPONSE = {
  status: 'success',
  request: 'a1b2c3d4-e5f6-7890-abcd-ef1234567890',
  receipt: 'r1234567890abcdef',
};

const MOCK_MESSAGES = [
  {
    id: 'msg-001',
    message: 'Test message from E2E',
    title: 'E2E 테스트',
    status: 'sent',
    priority: 0,
    sound: 'pushover',
    device: null,
    url: null,
    url_title: null,
    html: 0,
    receipt: 'r1234567890abcdef',
    sent_at: '2026-03-28T12:00:00Z',
    delivered_at: null,
    acknowledged_at: null,
    created_at: '2026-03-28T12:00:00Z',
  },
  {
    id: 'msg-002',
    message: '이전 메시지',
    title: null,
    status: 'delivered',
    priority: 0,
    sound: 'pushover',
    device: null,
    url: null,
    url_title: null,
    html: 0,
    receipt: 'r9876543210fedcba',
    sent_at: '2026-03-27T09:00:00Z',
    delivered_at: '2026-03-27T09:01:00Z',
    acknowledged_at: null,
    created_at: '2026-03-27T09:00:00Z',
  },
];

// API route mock 설정 헬퍼
async function setupApiMocks(page: import('@playwright/test').Page) {
  // 모든 Worker API 요청 mock
  await page.route('**/api/v1/**', async (route) => {
    const url = route.request().url();
    const method = route.request().method();

    if (url.includes('/api/v1/messages') && method === 'POST') {
      const body = route.request().postDataJSON();
      expect(body).toHaveProperty('user');
      expect(body).toHaveProperty('token');
      expect(body).toHaveProperty('message');

      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(MOCK_SEND_RESPONSE),
      });
    } else if (url.includes('/api/v1/messages') && method === 'GET') {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ status: 'success', messages: MOCK_MESSAGES }),
      });
    } else {
      await route.continue();
    }
  });
}

// localStorage에 설정 주입 헬퍼
async function injectSettings(page: import('@playwright/test').Page) {
  const settings = {
    pushover: {
      apiToken: 'azGDORePK8gMaC0QOYAMyEL1234567',
      userKey: 'uQiRzpo4DXghDmr9QzzfQu27cmVRsG',
    },
    worker: { url: MOCK_WORKER_URL, token: 'mock-worker-token-1234567890' },
    notification: { sound: 'pushover', device: 'all', priority: 0 },
  };
  await page.goto('/');
  await page.evaluate((s: typeof settings) => {
    localStorage.setItem('pushover-settings', btoa(JSON.stringify(s)));
  }, settings);
}

test.describe.serial('사용자 흐름: 설정 → 전송 → 이력', () => {
  test('1. Settings - API 설정 저장', async ({ page }) => {
    await page.goto('/settings');
    await expect(page.locator('h1')).toContainText('Settings');

    // API Token 입력 (30자 유효 포맷)
    const inputs = page.locator('input[type="password"]');
    await inputs.nth(0).fill('azGDORePK8gMaC0QOYAMyEL1234567');
    await inputs.nth(1).fill('uQiRzpo4DXghDmr9QzzfQu27cmVRsG');

    // 저장 클릭
    await page.getByRole('button', { name: '저장' }).click();

    // localStorage 검증
    const stored = await page.evaluate(() => {
      const raw = localStorage.getItem('pushover-settings');
      return raw ? JSON.parse(atob(raw)) : null;
    });
    expect(stored.pushover.apiToken).toBe('azGDORePK8gMaC0QOYAMyEL1234567');
    expect(stored.pushover.userKey).toBe('uQiRzpo4DXghDmr9QzzfQu27cmVRsG');
  });

  test('2. 메시지 전송 (mock API)', async ({ page }) => {
    await setupApiMocks(page);
    await injectSettings(page);

    await page.goto('/');
    await expect(page.locator('h1')).toContainText('PushOver Dashboard');

    // 모달 열기
    await page.getByRole('button', { name: '메시지 보내기' }).click();
    await expect(page.locator('h3')).toContainText('메시지 보내기');

    // 메시지 입력
    await page.fill('textarea[placeholder="전송할 메시지"]', 'Test message from E2E');

    // API 요청 가로채서 검증
    const apiRequest = page.waitForRequest(
      (req) => req.url().includes('/api/v1/messages') && req.method() === 'POST'
    );

    await page.getByRole('button', { name: '전송' }).click();

    // 요청 body 검증
    const request = await apiRequest;
    const body = request.postDataJSON();
    expect(body.message).toBe('Test message from E2E');
    expect(body.token).toBe('azGDORePK8gMaC0QOYAMyEL1234567');
    expect(body.user).toBe('uQiRzpo4DXghDmr9QzzfQu27cmVRsG');
    // 이미지 미첨부 시 image 필드 없어야 함
    expect(body.image).toBeUndefined();

    // 전송 완료 (alert 자동 처리)
    await expect(page.getByRole('button', { name: '전송 중...' })).toBeHidden({ timeout: 5000 });
  });

  test('2-1. 이미지 첨부 메시지 전송 (mock API)', async ({ page }) => {
    await setupApiMocks(page);
    await injectSettings(page);

    await page.goto('/');
    await expect(page.locator('h1')).toContainText('PushOver Dashboard');

    // 모달 열기
    await page.getByRole('button', { name: '메시지 보내기' }).click();
    await expect(page.locator('h3')).toContainText('메시지 보내기');

    // 메시지 입력
    await page.fill('textarea[placeholder="전송할 메시지"]', 'Image test message');

    // 이미지 파일 첨부
    const fileInput = page.locator('input[type="file"]');
    await fileInput.setInputFiles(path.resolve(__dirname, '../../tests/sample.jpg'));

    // 미리보기 이미지 렌더링 확인
    await expect(page.locator('img[alt="preview"]')).toBeVisible({ timeout: 3000 });

    // API 요청 가로채서 검증
    const apiRequest = page.waitForRequest(
      (req) => req.url().includes('/api/v1/messages') && req.method() === 'POST'
    );

    await page.getByRole('button', { name: '전송' }).click();

    // 요청 body에 image 필드(base64) 포함 확인
    const request = await apiRequest;
    const body = request.postDataJSON();
    expect(body.message).toBe('Image test message');
    expect(body.image).toBeDefined();
    expect(typeof body.image).toBe('string');
    expect(body.image.length).toBeGreaterThan(0);

    // 전송 완료
    await expect(page.getByRole('button', { name: '전송 중...' })).toBeHidden({ timeout: 5000 });
  });

  test('3. History - 메시지 이력 조회 (mock API)', async ({ page }) => {
    await setupApiMocks(page);
    await injectSettings(page);

    await page.goto('/history');

    await expect(page.locator('h1')).toContainText('Message History');

    // mock 데이터가 테이블에 렌더링되었는지 확인
    await expect(page.getByText('Test message from E2E')).toBeVisible();
    await expect(page.getByText('이전 메시지')).toBeVisible();

    // 상태 뱃지 확인 (exact match로 heading과 구분)
    await expect(page.getByText('sent', { exact: true })).toBeVisible();
    await expect(page.getByText('delivered', { exact: true })).toBeVisible();
  });
});
