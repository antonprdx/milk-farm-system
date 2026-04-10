import { api, buildQuery } from '$lib/api/client';

export interface MilkSummary {
	total_milk: number;
	count_days: number;
	avg_per_animal: number | null;
}

export interface ReproductionSummary {
	total_calvings: number;
	total_inseminations: number;
	total_pregnancies: number;
	total_heats: number;
	total_dry_offs: number;
}

export interface FeedSummary {
	total_feed_kg: number;
	total_visits: number;
}

export async function getMilkSummary(from?: string, till?: string) {
	return api<MilkSummary>(
		`/reports/milk-summary${buildQuery({ from_date: from, till_date: till })}`,
	);
}

export async function getReproductionSummary(from?: string, till?: string) {
	return api<ReproductionSummary>(
		`/reports/reproduction-summary${buildQuery({ from_date: from, till_date: till })}`,
	);
}

export async function getFeedSummary(from?: string, till?: string) {
	return api<FeedSummary>(
		`/reports/feed-summary${buildQuery({ from_date: from, till_date: till })}`,
	);
}

export function getExportUrl(
	type: 'milk' | 'reproduction' | 'feed',
	from?: string,
	till?: string,
): string {
	const base = import.meta.env.VITE_API_BASE || '/api/v1';
	return `${base}/reports/export/${type}${buildQuery({ from_date: from, till_date: till })}`;
}
