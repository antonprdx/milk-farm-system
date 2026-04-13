import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch }) => {
	try {
		const res = await fetch('/api/v1/locations', { credentials: 'include' });
		if (!res.ok) return { initialData: null };
		const data = await res.json();
		return { initialData: data };
	} catch {
		return { initialData: null };
	}
};
