import type { PageLoad } from './$types';
import { loadSimple } from '$lib/utils/ssr';

export const load: PageLoad = async ({ fetch }) => {
	return loadSimple(fetch, '/api/v1/locations');
};
