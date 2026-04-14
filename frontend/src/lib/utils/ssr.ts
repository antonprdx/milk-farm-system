// eslint-disable-next-line @typescript-eslint/no-explicit-any
export interface LoadResult {
	initialData: any;
	error?: string;
}

export async function loadPaginated(
	fetch: typeof globalThis.fetch,
	url: URL,
	endpoint: string,
	defaultPerPage = '50',
): Promise<LoadResult> {
	const page = url.searchParams.get('page') || '1';
	const perPage = url.searchParams.get('per_page') || defaultPerPage;
	const sep = endpoint.includes('?') ? '&' : '?';

	try {
		const res = await fetch(`${endpoint}${sep}page=${page}&per_page=${perPage}`, {
			credentials: 'include',
		});
		if (!res.ok) {
			return { initialData: null, error: `Ошибка загрузки (${res.status})` };
		}
		const data = await res.json();
		return { initialData: data };
	} catch {
		return { initialData: null, error: 'Не удалось подключиться к серверу' };
	}
}

export async function loadSimple(
	fetch: typeof globalThis.fetch,
	endpoint: string,
): Promise<LoadResult> {
	try {
		const res = await fetch(endpoint, { credentials: 'include' });
		if (!res.ok) {
			return { initialData: null, error: `Ошибка загрузки (${res.status})` };
		}
		const data = await res.json();
		return { initialData: data };
	} catch {
		return { initialData: null, error: 'Не удалось подключиться к серверу' };
	}
}
