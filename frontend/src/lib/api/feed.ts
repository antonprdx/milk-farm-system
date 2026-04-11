import { api, buildQuery } from './client';

export interface FeedDayAmount {
	id: number;
	animal_id: number;
	feed_date: string;
	feed_number: number;
	total: number;
	rest_feed: number | null;
}

export interface FeedVisit {
	id: number;
	animal_id: number;
	visit_datetime: string;
	feed_number: number | null;
	amount: number | null;
	duration_seconds: number | null;
}

export interface FeedType {
	id: number;
	number_of_feed_type: number;
	feed_type: string;
	name: string;
	description: string | null;
	dry_matter_percentage: number;
	stock_attention_level: number | null;
	price: number;
}

export interface FeedGroup {
	id: number;
	name: string;
	min_milk_yield: number | null;
	max_milk_yield: number | null;
	avg_milk_yield: number | null;
	avg_milk_fat: number | null;
	avg_milk_protein: number | null;
	avg_weight: number | null;
	max_robot_feed_types: number | null;
	max_feed_intake_robot: number | null;
	min_feed_intake_robot: number | null;
	number_of_cows: number | null;
}

export interface FeedFilter {
	animal_id?: string;
	from_date?: string;
	till_date?: string;
	page?: number;
	per_page?: number;
}

export function listDayAmounts(filter: FeedFilter = {}, signal?: AbortSignal) {
	return api<{ data: FeedDayAmount[]; total: number; page: number; per_page: number }>(
		`/feed/day-amounts${buildQuery(filter)}`,
		{ signal },
	);
}

export function listVisits(filter: FeedFilter = {}, signal?: AbortSignal) {
	return api<{ data: FeedVisit[]; total: number; page: number; per_page: number }>(
		`/feed/visits${buildQuery(filter)}`,
		{ signal },
	);
}

export function listTypes(signal?: AbortSignal) {
	return api<{ data: FeedType[] }>('/feed/types', { signal });
}

export function listGroups(signal?: AbortSignal) {
	return api<{ data: FeedGroup[] }>('/feed/groups', { signal });
}
