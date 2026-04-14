import { buildQuery as _buildQuery } from '$lib/utils/query';
import { auth } from '$lib/stores/auth';
import { goto } from '$app/navigation';
import { browser } from '$app/environment';

const API_BASE = import.meta.env.VITE_API_BASE || '/api/v1';

type RequestOptions = {
	method?: string;
	body?: unknown;
	signal?: AbortSignal;
	retries?: number;
};

const MAX_RETRIES = 3;
const BASE_DELAY = 500;

function backoff(attempt: number): number {
	return BASE_DELAY * Math.pow(2, attempt) + Math.random() * 200;
}

function sleep(ms: number): Promise<void> {
	return new Promise((r) => setTimeout(r, ms));
}

let refreshingPromise: Promise<boolean> | null = null;

async function tryRefresh(): Promise<boolean> {
	if (refreshingPromise) return refreshingPromise;
	refreshingPromise = (async () => {
		try {
			const res = await fetch(`${API_BASE}/auth/refresh`, {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				credentials: 'include',
			});
			return res.ok;
		} catch (e) {
			console.warn('Token refresh failed', e);
			return false;
		} finally {
			refreshingPromise = null;
		}
	})();
	return refreshingPromise;
}

export async function api<T>(path: string, opts: RequestOptions = {}): Promise<T> {
	const headers: Record<string, string> = {
		'Content-Type': 'application/json',
	};

	const maxRetries = opts.retries ?? MAX_RETRIES;

	for (let attempt = 0; ; attempt++) {
		const res = await fetch(`${API_BASE}${path}`, {
			method: opts.method ?? 'GET',
			headers,
			credentials: 'include',
			body: opts.body ? JSON.stringify(opts.body) : undefined,
			signal: opts.signal,
		});

		if (!res.ok) {
			if (res.status === 401 && browser) {
				const refreshed = await tryRefresh();
				if (refreshed) {
					const retryRes = await fetch(`${API_BASE}${path}`, {
						method: opts.method ?? 'GET',
						headers,
						credentials: 'include',
						body: opts.body ? JSON.stringify(opts.body) : undefined,
					});
					if (retryRes.ok) {
						return retryRes.json();
					}
				}
				auth.logout();
				goto('/auth/login');
				throw new Error('Сессия истекла. Войдите заново.');
			}

			if (res.status >= 500 && attempt < maxRetries) {
				await sleep(backoff(attempt));
				continue;
			}

			const err = await res.json().catch(() => ({ error: res.statusText }));
			throw new Error(err.error || res.statusText);
		}

		return res.json();
	}
}

export function del<T>(path: string) {
	return api<T>(path, { method: 'DELETE' });
}

export function post<T>(path: string, body: unknown) {
	return api<T>(path, { method: 'POST', body });
}

export function put<T>(path: string, body: unknown) {
	return api<T>(path, { method: 'PUT', body });
}

export function buildQuery(obj: object): string {
	return _buildQuery(obj);
}
