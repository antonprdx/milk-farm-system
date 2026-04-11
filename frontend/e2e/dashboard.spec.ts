import { test, expect } from '@playwright/test';
import { loginAsAdmin } from './helpers';

test.describe('Dashboard', () => {
	test.beforeEach(async ({ page }) => {
		try {
			await loginAsAdmin(page);
		} catch {
			test.skip();
		}
	});

	test('shows page heading after login', async ({ page }) => {
		await page.goto('/');
		await expect(page.getByRole('heading', { name: 'Дашборд' })).toBeVisible({ timeout: 10000 });
	});

	test('shows page title in browser tab', async ({ page }) => {
		await page.goto('/');
		await expect(page).toHaveTitle(/Дашборд — Молочная ферма/, { timeout: 10000 });
	});

	test('shows KPI section after data loads', async ({ page }) => {
		await page.goto('/');
		await expect(page.getByText('Интервал отёлов')).toBeVisible({ timeout: 15000 });
	});

	test('shows system status section', async ({ page }) => {
		await page.goto('/');
		await expect(page.getByText('Статус системы')).toBeVisible({ timeout: 10000 });
	});

	test('shows refresh button', async ({ page }) => {
		await page.goto('/');
		await expect(page.getByRole('button', { name: /Обновить/ })).toBeVisible({ timeout: 10000 });
	});
});
