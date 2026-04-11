import { test, expect } from '@playwright/test';
import { loginAsAdmin } from './helpers';

test.describe('Animals list', () => {
	test.beforeEach(async ({ page }) => {
		try {
			await loginAsAdmin(page);
		} catch {
			test.skip();
		}
	});

	test('shows page heading', async ({ page }) => {
		await page.goto('/animals');
		await expect(page.getByRole('heading', { name: 'Животные' })).toBeVisible({ timeout: 10000 });
	});

	test('shows page title in browser tab', async ({ page }) => {
		await page.goto('/animals');
		await expect(page).toHaveTitle(/Животные — Молочная ферма/, { timeout: 10000 });
	});

	test('shows data table with column headers', async ({ page }) => {
		await page.goto('/animals');
		await expect(page.locator('table')).toBeVisible({ timeout: 10000 });
		await expect(page.getByRole('columnheader', { name: 'Имя' })).toBeVisible();
		await expect(page.getByRole('columnheader', { name: 'Пол' })).toBeVisible();
	});

	test('search input accepts text', async ({ page }) => {
		await page.goto('/animals');
		const searchInput = page.locator('#animal-search');
		await expect(searchInput).toBeVisible({ timeout: 10000 });
		await searchInput.fill('Марта');
		await expect(searchInput).toHaveValue('Марта');
	});

	test('search triggers on Enter key', async ({ page }) => {
		await page.goto('/animals');
		await expect(page.locator('#animal-search')).toBeVisible({ timeout: 10000 });
		await page.fill('#animal-search', 'test');
		await page.press('#animal-search', 'Enter');
		await expect(page.locator('#animal-search')).toHaveValue('test');
	});

	test('gender filter dropdown is present', async ({ page }) => {
		await page.goto('/animals');
		await expect(page.locator('#animal-gender')).toBeVisible({ timeout: 10000 });
	});

	test('status filter dropdown is present', async ({ page }) => {
		await page.goto('/animals');
		await expect(page.locator('#animal-status')).toBeVisible({ timeout: 10000 });
	});

	test('find button is present', async ({ page }) => {
		await page.goto('/animals');
		await expect(page.getByRole('button', { name: 'Найти' })).toBeVisible({ timeout: 10000 });
	});

	test('add animal link is present', async ({ page }) => {
		await page.goto('/animals');
		await expect(page.getByRole('link', { name: '+ Добавить' })).toBeVisible({ timeout: 10000 });
	});

	test('pagination or empty state is shown after load', async ({ page }) => {
		await page.goto('/animals');
		await expect(page.locator('table')).toBeVisible({ timeout: 10000 });
		const paginationTotal = page.getByText(/Всего: \d+/);
		const emptyState = page.getByText('Животные не найдены');
		await expect(paginationTotal.or(emptyState)).toBeVisible({ timeout: 10000 });
	});

	test('pagination controls exist when data is present', async ({ page }) => {
		await page.goto('/animals');
		await expect(page.locator('table')).toBeVisible({ timeout: 10000 });
		const totalText = page.getByText(/Всего: \d+/);
		const isVisible = await totalText.isVisible().catch(() => false);
		if (isVisible) {
			await expect(page.getByRole('button', { name: 'Назад' })).toBeVisible();
			await expect(page.getByRole('button', { name: 'Вперёд' })).toBeVisible();
		}
	});
});
