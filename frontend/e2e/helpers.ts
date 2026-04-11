import type { Page } from '@playwright/test';

export async function loginAsAdmin(page: Page): Promise<void> {
	const apiUrl = process.env.E2E_API_URL || 'http://localhost:3000/api/v1';
	const res = await page.request.post(`${apiUrl}/auth/login`, {
		data: { username: 'admin', password: 'admin' },
	});
	if (!res.ok()) throw new Error(`Login failed: ${res.status()}`);
	await page.addInitScript(() => {
		localStorage.setItem('username', 'admin');
		localStorage.setItem('role', 'admin');
		localStorage.setItem('mustChangePassword', 'false');
	});
}
