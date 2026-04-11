import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch, url }) => {
	const page = url.searchParams.get('page') || '1';
	const perPage = url.searchParams.get('per_page') || '50';

	try {
		const res = await fetch(`/api/v1/bulk-tank?page=${page}&per_page=${perPage}`, {
			credentials: 'include',
		});
		if (!res.ok) return { initialData: null };
		const data = await res.json();
		return { initialData: data };
	} catch {
		return { initialData: null };
	}
};
