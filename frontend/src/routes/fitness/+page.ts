import type { PageLoad } from './$types';
import { loadPaginated } from '$lib/utils/ssr';

export const load: PageLoad = async ({ fetch, url }) => {
	return loadPaginated(fetch, url, '/api/v1/fitness/activities');
};
