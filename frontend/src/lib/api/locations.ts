import { api, buildQuery } from './client';

export interface Location {
	id: number;
	name: string;
	location_type: string | null;
	created_at: string;
}

export interface LocationFilter {
	page?: number;
	per_page?: number;
}

export function listLocations(filter: LocationFilter = {}, signal?: AbortSignal) {
	return api<{ data: Location[]; total: number }>(`/locations${buildQuery(filter)}`, { signal });
}
