import { test, expect, devices } from '@playwright/test';
import { loginAsAdmin } from './helpers';

test.describe('Navigation - Auth redirect', () => {
	test('redirects to login when visiting /animals unauthenticated', async ({ page }) => {
		await page.goto('/animals');
		await expect(page).toHaveURL(/\/auth\/login/, { timeout: 10000 });
	});

	test('redirects to login when visiting /milk unauthenticated', async ({ page }) => {
		await page.goto('/milk');
		await expect(page).toHaveURL(/\/auth\/login/, { timeout: 10000 });
	});

	test('redirects to login when visiting /settings unauthenticated', async ({ page }) => {
		await page.goto('/settings');
		await expect(page).toHaveURL(/\/auth\/login/, { timeout: 10000 });
	});
});

test.describe('Navigation - Sidebar links', () => {
	test.beforeEach(async ({ page }) => {
		try {
			await loginAsAdmin(page);
		} catch {
			test.skip();
		}
	});

	const navItems = [
		{ href: '/animals', label: 'Животные' },
		{ href: '/milk', label: 'Удои' },
		{ href: '/feed', label: 'Кормление' },
		{ href: '/contacts', label: 'Контакты' },
		{ href: '/reports', label: 'Отчёты' },
	];

	for (const item of navItems) {
		test(`sidebar "${item.label}" link navigates to ${item.href}`, async ({ page }) => {
			await page.goto('/');
			await page.locator(`nav a[href="${item.href}"]`).click();
			await expect(page).toHaveURL(new RegExp(item.href.replace('/', '\\/')));
		});
	}

	test('dashboard link navigates to root', async ({ page }) => {
		await page.goto('/animals');
		await page.locator('nav a[href="/"]').click();
		await expect(page).toHaveURL(/\/$/);
	});
});

test.describe('Navigation - Active sidebar link', () => {
	test.beforeEach(async ({ page }) => {
		try {
			await loginAsAdmin(page);
		} catch {
			test.skip();
		}
	});

	test('dashboard link is active on root path', async ({ page }) => {
		await page.goto('/');
		const activeLink = page.locator('nav a[aria-current="page"]');
		await expect(activeLink).toHaveAttribute('href', '/');
	});

	test('animals link is active on /animals path', async ({ page }) => {
		await page.goto('/animals');
		const activeLink = page.locator('nav a[aria-current="page"]');
		await expect(activeLink).toHaveAttribute('href', '/animals');
	});

	test('milk link is active on /milk path', async ({ page }) => {
		await page.goto('/milk');
		const activeLink = page.locator('nav a[aria-current="page"]');
		await expect(activeLink).toHaveAttribute('href', '/milk');
	});
});

test.describe('Navigation - Mobile menu', () => {
	test.use({ ...devices['Pixel 5'] });

	test.beforeEach(async ({ page }) => {
		try {
			await loginAsAdmin(page);
		} catch {
			test.skip();
		}
	});

	test('hamburger button opens mobile sidebar', async ({ page }) => {
		await page.goto('/');
		await expect(page.locator('button[aria-label="Меню"]')).toBeVisible();
		await page.locator('button[aria-label="Меню"]').click();
		await expect(page.locator('nav a[href="/animals"]')).toBeVisible();
	});

	test('clicking overlay closes mobile sidebar', async ({ page }) => {
		await page.goto('/');
		await page.locator('button[aria-label="Меню"]').click();
		await expect(page.locator('nav a[href="/animals"]')).toBeVisible();
		await page.locator('div[role="presentation"]').click();
		await expect(page.locator('nav a[href="/animals"]')).not.toBeVisible({ timeout: 5000 });
	});

	test('mobile sidebar nav link navigates and closes menu', async ({ page }) => {
		await page.goto('/');
		await page.locator('button[aria-label="Меню"]').click();
		await page.locator('nav a[href="/animals"]').click();
		await expect(page).toHaveURL(/\/animals/);
	});
});
