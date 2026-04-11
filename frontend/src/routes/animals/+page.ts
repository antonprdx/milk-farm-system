import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch, url }) => {
	const page = url.searchParams.get('page') || '1';
	const perPage = url.searchParams.get('per_page') || '20';
	const search = url.searchParams.get('search') || '';
	const gender = url.searchParams.get('gender') || '';
	const active = url.searchParams.get('active') || 'true';

	const params = new URLSearchParams({ page, per_page: perPage });
	if (search) params.set('search', search);
	if (gender) params.set('gender', gender);
	if (active) params.set('active', active);

	try {
		const res = await fetch(`/api/v1/animals?${params}`, { credentials: 'include' });
		if (!res.ok) return { initialData: null };
		const data = await res.json();
		return { initialData: data };
	} catch {
		return { initialData: null };
	}
};
