import { api, post, put, del, buildQuery } from './client';

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

export interface CreateFeedType {
	number_of_feed_type: number;
	feed_type: string;
	name: string;
	description?: string;
	dry_matter_percentage: number;
	stock_attention_level?: number;
	price: number;
}

export interface UpdateFeedType {
	number_of_feed_type?: number;
	feed_type?: string;
	name?: string;
	description?: string;
	dry_matter_percentage?: number;
	stock_attention_level?: number;
	price?: number;
}

export interface CreateFeedGroup {
	name: string;
	min_milk_yield?: number;
	max_milk_yield?: number;
	avg_milk_yield?: number;
	avg_milk_fat?: number;
	avg_milk_protein?: number;
	avg_weight?: number;
	max_robot_feed_types?: number;
	max_feed_intake_robot?: number;
	min_feed_intake_robot?: number;
	number_of_cows?: number;
}

export interface UpdateFeedGroup {
	name?: string;
	min_milk_yield?: number;
	max_milk_yield?: number;
	avg_milk_yield?: number;
	avg_milk_fat?: number;
	avg_milk_protein?: number;
	avg_weight?: number;
	max_robot_feed_types?: number;
	max_feed_intake_robot?: number;
	min_feed_intake_robot?: number;
	number_of_cows?: number;
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

export function createFeedType(data: CreateFeedType) {
	return post<{ data: FeedType }>('/feed/types', data);
}

export function updateFeedType(id: number, data: UpdateFeedType) {
	return put<{ data: FeedType }>(`/feed/types/${id}`, data);
}

export function deleteFeedType(id: number) {
	return del<{ message: string }>(`/feed/types/${id}`);
}

export function createFeedGroup(data: CreateFeedGroup) {
	return post<{ data: FeedGroup }>('/feed/groups', data);
}

export function updateFeedGroup(id: number, data: UpdateFeedGroup) {
	return put<{ data: FeedGroup }>(`/feed/groups/${id}`, data);
}

export function deleteFeedGroup(id: number) {
	return del<{ message: string }>(`/feed/groups/${id}`);
}
