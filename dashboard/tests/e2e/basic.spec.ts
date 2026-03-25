import { test, expect } from '@playwright/test';

test.describe('PushOver Dashboard', () => {
  test('메인 페이지 로드', async ({ page }) => {
    await page.goto('/');
    
    await expect(page.locator('h1')).toContainText('PushOver Dashboard');
    await expect(page.locator('text=메시지 보내기')).toBeVisible();
  });

  test('메시지 전송 모달 열기', async ({ page }) => {
    await page.goto('/');
    
    await page.click('text=메시지 보내기');
    
    await expect(page.locator('text=메시지 보내기')).toBeVisible();
    await expect(page.locator('text=제목 (선택)')).toBeVisible();
    await expect(page.locator('text=메시지')).toBeVisible();
  });

  test('메시지 전송', async ({ page }) => {
    await page.goto('/');
    
    await page.click('text=메시지 보내기');
    await page.fill('textarea[placeholder="전송할 메시지"]', 'Test message from E2E');
    await page.click('text=전송');
    
    // TODO: 실제 API 호출 후 검증
    await expect(page.locator('text=전송 중...')).toBeHidden();
  });

  test('History 페이지 이동', async ({ page }) => {
    await page.goto('/');
    
    await page.click('text=History');
    
    await expect(page).toHaveURL(/\/history/);
    await expect(page.locator('h1')).toContainText('Message History');
  });
});

test.describe('Settings', () => {
  test('API 키 설정', async ({ page }) => {
    await page.goto('/settings');
    
    await expect(page.locator('h1')).toContainText('Settings');
    await expect(page.locator('text=API Key')).toBeVisible();
    
    await page.fill('input[type="password"]', 'test-api-key-123');
    await page.click('text=Save Settings');
    
    // TODO: 로컬 스토리지 저장 확인
  });
});
