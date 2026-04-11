import { test, expect } from '@playwright/test';

test.describe('Authentication', () => {
	test('redirects to login when not authenticated', async ({ page }) => {
		await page.goto('/');
		await expect(page).toHaveURL(/\/auth\/login/, { timeout: 10000 });
	});

	test('shows login form', async ({ page }) => {
		await page.goto('/auth/login');
		await expect(page.locator('input[name="username"]')).toBeVisible();
		await expect(page.locator('input[name="password"]')).toBeVisible();
		await expect(page.locator('button[type="submit"]')).toBeVisible();
	});

	test('shows error on invalid credentials', async ({ page }) => {
		await page.goto('/auth/login');
		await page.fill('input[name="username"]', 'nonexistent');
		await page.fill('input[name="password"]', 'wrong');
		await page.click('button[type="submit"]');
		await expect(page.locator('text=/неверн|invalid|ошибка/i')).toBeVisible({ timeout: 5000 });
	});
});

test.describe('Dashboard (requires API)', () => {
	test.beforeEach(async ({ page }) => {
		const apiUrl = process.env.E2E_API_URL || 'http://localhost:3000/api/v1';
		const res = await page.request.post(`${apiUrl}/auth/login`, {
			data: { username: 'admin', password: 'admin' },
		});
		if (!res.ok()) test.skip();
	});

	test('dashboard page has title', async ({ page }) => {
		await page.goto('/');
		const title = await page.title();
		expect(title).toContain('Молочная ферма');
	});
});

test.describe('Navigation', () => {
	test('skip-to-content link exists', async ({ page }) => {
		await page.goto('/auth/login');
		const skipLink = page.locator('a[href="#main-content"]');
		await expect(skipLink).toBeAttached();
	});
});
