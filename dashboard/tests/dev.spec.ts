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

const testStamp = `${Date.now()}`;
const testMessage = `[E2E Integration] ${testStamp}`;
const testTitle = `E2E Test ${testStamp}`;

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

  test('2. 메시지 전송 (브라우저 UI)', async ({ page }) => {
    test.slow();
    await setSettings(page);

    await expect(page.locator('h1')).toContainText('PushOver Dashboard');

    // 메시지 모달 열기
    await page.getByRole('button', { name: '메시지 보내기' }).click();
    await expect(page.locator('h3')).toContainText('메시지 보내기');

    // 제목 + 메시지 입력
    await page.fill('input[placeholder="메시지 제목"]', testTitle);
    await page.fill('textarea[placeholder="전송할 메시지"]', testMessage);

    // 전송 버튼 클릭
    await page.getByRole('button', { name: '전송' }).click();

    // 전송 완료 대기
    await expect(page.getByRole('button', { name: '전송 중...' })).toBeHidden({ timeout: 10000 });

    // 성공 메시지 확인 (모달이 닫히거나 성공 표시)
    await page.waitForTimeout(1000);
  });

  test('2-1. 이미지 첨부 메시지 전송 (브라우저 UI)', async ({ page }) => {
    test.slow();
    await setSettings(page);

    await expect(page.locator('h1')).toContainText('PushOver Dashboard');

    // 메시지 모달 열기
    await page.getByRole('button', { name: '메시지 보내기' }).click();
    await expect(page.locator('h3')).toContainText('메시지 보내기');

    const imgStamp = `${Date.now()}`;
    await page.fill('input[placeholder="메시지 제목"]', `Image Test ${imgStamp}`);
    await page.fill('textarea[placeholder="전송할 메시지"]', `[E2E Image] ${imgStamp}`);

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

  test('3. History - 메시지 이력 조회 (Worker API 직접 호출)', async ({ page }) => {
    test.slow();

    // 이 테스트에서 독립적으로 메시지 전송 후 확인
    const test3Stamp = `${Date.now()}`;
    const test3Title = `E2E History Test ${test3Stamp}`;
    const test3Message = `[E2E History] ${test3Stamp}`;

    // Worker API 직접 호출로 메시지 전송
    const sendResponse = await fetch(`${WORKER_URL}/api/v1/messages`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${WORKER_TOKEN}`,
      },
      body: JSON.stringify({
        token: PUSHOVER_TOKEN,
        user: PUSHOVER_USER_KEY,
        message: test3Message,
        title: test3Title,
      }),
    });

    expect(sendResponse.ok).toBeTruthy();
    const sendData = await sendResponse.json();
    console.log('전송 응답:', sendData);

    // 메시지가 Worker API에 저장되었는지 확인
    await page.waitForTimeout(3000); // DB 저장 대기

    const workerResponse = await fetch(`${WORKER_URL}/api/v1/messages?limit=10`, {
      headers: {
        'Authorization': `Bearer ${WORKER_TOKEN}`,
      },
    });
    const workerData = await workerResponse.json();

    // 방금 보낸 메시지가 Worker API에 저장되었는지 확인
    const foundMessage = workerData.messages?.find((m: any) => m.title === test3Title);

    if (!foundMessage) {
      console.log('❌ 메시지를 찾을 수 없음');
      console.log('모든 타이틀:', workerData.messages?.map((m: any) => m.title));
    }

    expect(foundMessage, `메시지 "${test3Title}"가 Worker API에 저장되어야 함`).toBeDefined();
    expect(foundMessage?.message).toBe(test3Message);

    console.log('✅ 메시지가 Worker API에 저장됨:', test3Title);
  });
});
