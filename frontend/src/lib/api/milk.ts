import { api, post, put, del, buildQuery } from './client';

export interface MilkDayProduction {
	id: number;
	animal_id: number;
	date: string;
	milk_amount: number | null;
	avg_amount: number | null;
	avg_weight: number | null;
	isk: number | null;
	created_at: string;
}

export interface MilkVisit {
	id: number;
	animal_id: number;
	visit_datetime: string;
	milk_amount: number | null;
	duration_seconds: number | null;
	milk_destination: number | null;
	created_at: string;
}

export interface MilkQuality {
	id: number;
	animal_id: number;
	date: string;
	milk_amount: number | null;
	avg_amount: number | null;
	avg_weight: number | null;
	isk: number | null;
	fat_percentage: number | null;
	protein_percentage: number | null;
	lactose_percentage: number | null;
	scc: number | null;
	milkings: number | null;
	refusals: number | null;
	failures: number | null;
	created_at: string;
}

export interface CreateMilkDayProduction {
	animal_id: number;
	date: string;
	milk_amount?: number;
	avg_amount?: number;
	avg_weight?: number;
	isk?: number;
}

export interface UpdateMilkDayProduction {
	date?: string;
	milk_amount?: number;
	avg_amount?: number;
	avg_weight?: number;
	isk?: number;
}

export interface MilkFilter {
	animal_id?: string;
	from_date?: string;
	till_date?: string;
	page?: number;
	per_page?: number;
}

export function listProductions(filter: MilkFilter = {}) {
	return api<{ data: MilkDayProduction[]; total: number }>(
		`/milk/day-productions${buildQuery(filter)}`,
	);
}

export function createProduction(data: CreateMilkDayProduction) {
	return post<{ data: MilkDayProduction }>('/milk/day-productions', data);
}

export function updateProduction(id: number, data: UpdateMilkDayProduction) {
	return put<{ data: MilkDayProduction }>(`/milk/day-productions/${id}`, data);
}

export function deleteProduction(id: number) {
	return del<{ message: string }>(`/milk/day-productions/${id}`);
}

export function listVisits(filter: MilkFilter = {}) {
	return api<{ data: MilkVisit[]; total: number }>(`/milk/visits${buildQuery(filter)}`);
}

export function listQuality(filter: MilkFilter = {}) {
	return api<{ data: MilkQuality[]; total: number }>(`/milk/quality${buildQuery(filter)}`);
}
