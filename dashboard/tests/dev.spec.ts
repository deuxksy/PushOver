import { test, expect } from '@playwright/test';
import path from 'path';
import fs from 'fs';
import { config } from 'dotenv';

// .env 파일에서 환경변수 로드
config({ path: path.resolve(__dirname, '../../.env') });

const WORKER_URL = process.env.CLOUDFLARE_WORKER_URL || '';
const WORKER_TOKEN = process.env.CLOUDFLARE_WORKER_TOKEN || '';
const PUSHOVER_TOKEN = process.env.PUSHOVER_API_TOKEN || '';
const PUSHOVER_USER_KEY = process.env.PUSHOVER_USER_KEY || '';

test.skip(!WORKER_URL || !WORKER_TOKEN || !PUSHOVER_TOKEN || !PUSHOVER_USER_KEY,
  '환경변수 필요: .env에 CLOUDFLARE_WORKER_URL, CLOUDFLARE_WORKER_TOKEN, PUSHOVER_API_TOKEN, PUSHOVER_USER_KEY 설정');

test.describe.serial('실제 API 연동 테스트 (브라우저)', () => {
  // Helper 함수: localStorage 설정
  async function setSettings(page: any) {
    const settings = {
      pushover: {
        apiToken: PUSHOVER_TOKEN,
        userKey: PUSHOVER_USER_KEY,
      },
      worker: {
        url: WORKER_URL,
        token: WORKER_TOKEN,
        webhookSecret: '',
      },
      notification: {
        sound: 'pushover',
        device: '',
        priority: 0,
      },
    };

    await page.goto('/');
    await page.evaluate((settings: any) => {
      localStorage.setItem('pushover-settings', btoa(JSON.stringify(settings)));
    }, settings);
    await page.reload();
  }

  test('1. Worker 헬스체크', async ({ page }) => {
    test.slow();
    const res = await fetch(`${WORKER_URL}/health`);
    expect(res.ok).toBeTruthy();
    expect(await res.text()).toBe('OK');
  });

  test('2. 이미지 첨부 메시지 전송 (브라우저 UI)', async ({ page }) => {
    test.slow();
    await setSettings(page);

    await expect(page.locator('h1')).toContainText('PushOver Dashboard');

    // 메시지 모달 열기
    await page.getByRole('button', { name: '메시지 보내기' }).click();
    await expect(page.locator('h3')).toContainText('메시지 보내기');

    const testId = `${Date.now()}`;
    const testLabel = process.env.TEST_NAME || 'dashboard-dev';
    await page.fill('input[placeholder="메시지 제목"]', testLabel);
    await page.fill('textarea[placeholder="전송할 메시지"]', `[${testLabel}] ${testId}`);

    // 이미지 파일 첨부
    const sampleImage = path.resolve(__dirname, '../../tests/sample.jpg');
    expect(fs.existsSync(sampleImage), 'tests/sample.jpg 가 존재해야 함').toBeTruthy();
    const fileInput = page.locator('input[type="file"]');
    await fileInput.setInputFiles(sampleImage);

    // 미리보기 확인
    await expect(page.locator('img[alt="preview"]')).toBeVisible({ timeout: 3000 });

    // 전송
    await page.getByRole('button', { name: '전송' }).click();
    await expect(page.getByRole('button', { name: '전송 중...' })).toBeHidden({ timeout: 15000 });
    await page.waitForTimeout(1000);
  });

});
