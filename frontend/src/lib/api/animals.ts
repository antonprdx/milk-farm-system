import { api, del, post, put, buildQuery } from './client';

export interface Animal {
	id: number;
	life_number: string | null;
	name: string | null;
	user_number: number | null;
	gender: 'male' | 'female';
	birth_date: string;
	hair_color_code: string | null;
	father_life_number: string | null;
	mother_life_number: string | null;
	description: string | null;
	ucn_number: string | null;
	use_as_sire: boolean | null;
	location: string | null;
	group_number: number | null;
	keep: boolean | null;
	gestation: number | null;
	responder_number: string | null;
	active: boolean;
	created_at: string;
	updated_at: string;
}

export interface AnimalListResponse {
	data: Animal[];
	total: number;
}

export interface AnimalFilter {
	search?: string;
	life_number?: string;
	ucn_number?: string;
	active?: boolean;
	gender?: 'male' | 'female';
	page?: number;
	per_page?: number;
}

export interface CreateAnimal {
	life_number?: string;
	name?: string;
	user_number?: number;
	gender: 'male' | 'female';
	birth_date: string;
	hair_color_code?: string;
	father_life_number?: string;
	mother_life_number?: string;
	description?: string;
	ucn_number?: string;
	use_as_sire?: boolean;
	location?: string;
	group_number?: number;
	keep?: boolean;
	gestation?: number;
	responder_number?: string;
}

export interface UpdateAnimal {
	name?: string;
	hair_color_code?: string;
	description?: string;
	ucn_number?: string;
	use_as_sire?: boolean;
	location?: string;
	group_number?: number;
	keep?: boolean;
	gestation?: number;
	responder_number?: string;
	active?: boolean;
}

export function listAnimals(filter: AnimalFilter = {}, signal?: AbortSignal) {
	return api<AnimalListResponse>(`/animals${buildQuery(filter)}`, { signal });
}

export function getAnimal(id: number, signal?: AbortSignal) {
	return api<{ data: Animal }>(`/animals/${id}`, { signal });
}

export function createAnimal(data: CreateAnimal) {
	return post<{ data: Animal }>('/animals', data);
}

export function updateAnimal(id: number, data: UpdateAnimal) {
	return put<{ data: Animal }>(`/animals/${id}`, data);
}

export function deleteAnimal(id: number) {
	return del<{ message: string }>(`/animals/${id}`);
}

export function batchDeactivateAnimals(ids: number[]) {
	return post<{ message: string; count: number }>('/animals/batch/deactivate', { ids });
}

export interface TimelineEvent {
	date: string;
	event_type: string;
	description: string;
}

export interface TimelineResponse {
	data: TimelineEvent[];
	total: number;
	page: number;
	per_page: number;
}

export function getAnimalTimeline(id: number, page = 1, perPage = 50, signal?: AbortSignal) {
	return api<TimelineResponse>(`/animals/${id}/timeline?page=${page}&per_page=${perPage}`, {
		signal,
	});
}

export interface MilkDataPoint {
	date: string;
	amount: number;
}

export interface SccDataPoint {
	date: string;
	scc: number;
}

export interface LatestMetrics {
	avg_milk_30d: number | null;
	last_scc: number | null;
	avg_weight_30d: number | null;
	avg_isk_30d: number | null;
}

export interface ReproductionSummary {
	last_calving_date: string | null;
	total_inseminations: number;
	expected_calving_date: string | null;
	is_pregnant: boolean;
	lactation_number: number | null;
	days_in_milk: number | null;
	is_dry: boolean;
}

export interface AnimalStats {
	milk_production_30d: MilkDataPoint[];
	scc_trend_90d: SccDataPoint[];
	latest_metrics: LatestMetrics;
	reproduction: ReproductionSummary;
}

export function getAnimalStats(id: number, signal?: AbortSignal) {
	return api<AnimalStats>(`/animals/${id}/stats`, { signal });
}
