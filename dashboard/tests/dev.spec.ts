import { test, expect } from '@playwright/test';
import path from 'path';
import { config } from 'dotenv';

// .env.test 파일에서 환경변수 로드
config({ path: path.resolve(__dirname, '../../.env.test') });

const WORKER_URL = process.env.WORKER_URL || '';
const WORKER_TOKEN = process.env.WORKER_TOKEN || '';
const PUSHOVER_TOKEN = process.env.PUSHOVER_TOKEN || '';
const PUSHOVER_USER_KEY = process.env.PUSHOVER_USER_KEY || '';

test.skip(!WORKER_URL || !WORKER_TOKEN || !PUSHOVER_TOKEN || !PUSHOVER_USER_KEY,
  '환경변수 필요: .env.test에 WORKER_URL, WORKER_TOKEN, PUSHOVER_TOKEN, PUSHOVER_USER_KEY 설정');

const testStamp = `${Date.now()}`;
const testMessage = `[E2E Integration] ${testStamp}`;
const testTitle = `E2E Test ${testStamp}`;

// localStorage에 실제 설정 주입
async function injectRealSettings(page: any) {
  await page.goto('/');
  await page.evaluate((s: any) => {
    localStorage.setItem('pushover-settings', btoa(JSON.stringify(s)));
  }, {
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
  });
}

test.describe.serial('실제 API 연동 테스트 (브라우저)', () => {
  test('1. Worker 헬스체크', async () => {
    test.slow();
    const res = await fetch(`${WORKER_URL}/health`);
    expect(res.ok).toBeTruthy();
    expect(await res.text()).toBe('OK');
  });

  test('2. 메시지 전송 (브라우저 UI)', async ({ page }) => {
    test.slow();
    await injectRealSettings(page);

    // 홈페이지로 이동
    await page.goto('/');
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

  test('3. History - 메시지 이력 조회 (실제 API)', async ({ page }) => {
    test.slow();
    await injectRealSettings(page);

    // History 페이지로 이동
    await page.goto('/history');
    await expect(page.locator('h1')).toContainText('Message History');

    // 로딩 완료 대기
    await expect(page.getByText('Loading...')).toBeHidden({ timeout: 15000 });

    // 디버그: 페이지 텍스트 캡처
    const bodyText = await page.locator('main').textContent();
    console.log('=== History page content ===');
    console.log(bodyText);
    console.log(`=== Looking for: "${testTitle}" ===`);

    // 방금 보낸 메시지가 테이블에 표시되는지 확인
    await expect(page.getByText(testTitle)).toBeVisible({ timeout: 15000 });
    await expect(page.getByText(testMessage)).toBeVisible();
  });
});
