import { api, buildQuery } from './client';

export interface GrazingData {
	id: number;
	date: string;
	animal_count: number | null;
	pasture_time: number | null;
	lactation_period: string | null;
}

export interface GrazingFilter {
	from_date?: string;
	till_date?: string;
	page?: number;
	per_page?: number;
}

export function listGrazing(filter: GrazingFilter = {}, signal?: AbortSignal) {
	return api<{ data: GrazingData[]; total: number; page: number; per_page: number }>(
		`/grazing${buildQuery(filter)}`,
		{ signal },
	);
}
