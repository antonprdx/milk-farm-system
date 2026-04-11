import { api, buildQuery } from './client';

export interface Activity {
	id: number;
	animal_id: number;
	activity_datetime: string;
	activity_counter: number | null;
	heat_attention: boolean | null;
}

export interface Rumination {
	id: number;
	animal_id: number;
	date: string;
	eating_seconds: number | null;
	rumination_minutes: number | null;
}

export interface FitnessFilter {
	animal_id?: string;
	from_date?: string;
	till_date?: string;
	page?: number;
	per_page?: number;
}

export function listActivities(filter: FitnessFilter = {}, signal?: AbortSignal) {
	return api<{ data: Activity[]; total: number; page: number; per_page: number }>(
		`/fitness/activities${buildQuery(filter)}`,
		{ signal },
	);
}

export function listRuminations(filter: FitnessFilter = {}, signal?: AbortSignal) {
	return api<{ data: Rumination[]; total: number; page: number; per_page: number }>(
		`/fitness/ruminations${buildQuery(filter)}`,
		{ signal },
	);
}
