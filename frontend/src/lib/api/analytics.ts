import { api, buildQuery } from './client';

export interface LactationAvg {
	lac: number;
	avg_milk: number | null;
}

export interface CullingRiskEntry {
	animal_id: number;
	name: string | null;
	life_number: string | null;
	score: number;
	reasons: string[];
}

export interface KpiResponse {
	avg_calving_interval_days: number | null;
	conception_rate_pct: number | null;
	avg_milk_by_lactation: LactationAvg[];
	feed_efficiency: number | null;
	avg_days_to_first_ai: number | null;
	avg_scc: number | null;
	refusal_rate_pct: number | null;
	culling_risk: CullingRiskEntry[];
}

export interface Alert {
	alert_type: string;
	severity: string;
	animal_id: number | null;
	animal_name: string | null;
	message: string;
	value: string;
}

export interface AlertsResponse {
	alerts: Alert[];
}

export interface DailyMilkPoint {
	date: string;
	total_milk: number | null;
	cow_count: number | null;
}

export interface ForecastPoint {
	date: string;
	predicted: number;
	lower: number;
	upper: number;
}

export interface MilkTrendResponse {
	daily: DailyMilkPoint[];
	forecast: ForecastPoint[];
	trend_direction: string;
}

export interface ExpectedCalving {
	animal_id: number;
	name: string | null;
	life_number: string | null;
	insemination_date: string | null;
	expected_date: string;
	days_left: number;
}

export interface ExpectedHeat {
	animal_id: number;
	name: string | null;
	life_number: string | null;
	last_heat: string;
	expected_next: string;
	days_until: number;
	overdue: boolean;
}

export interface DryOffRecommendation {
	animal_id: number;
	name: string | null;
	life_number: string | null;
	expected_calving: string;
	recommended_dry_off: string;
	days_until_dry_off: number;
}

export interface ReproductionForecastResponse {
	expected_calvings: ExpectedCalving[];
	expected_heats: ExpectedHeat[];
	dry_off_recommendations: DryOffRecommendation[];
}

export interface FeedForecastResponse {
	weekly_feed_kg: number | null;
	predicted_next_week_kg: number;
	avg_per_cow_day_kg: number | null;
	milk_per_feed: number | null;
}

export function getKpi(signal?: AbortSignal) {
	return api<KpiResponse>('/analytics/kpi', { signal });
}

export function getAlerts(signal?: AbortSignal) {
	return api<AlertsResponse>('/analytics/alerts', { signal });
}

export function getMilkTrend(days?: number, forecastDays?: number, signal?: AbortSignal) {
	const params: Record<string, string> = {};
	if (days) params.days = String(days);
	if (forecastDays) params.forecast_days = String(forecastDays);
	return api<MilkTrendResponse>(`/analytics/milk-trend${buildQuery(params)}`, { signal });
}

export function getReproductionForecast(signal?: AbortSignal) {
	return api<ReproductionForecastResponse>('/analytics/reproduction-forecast', { signal });
}

export function getFeedForecast(signal?: AbortSignal) {
	return api<FeedForecastResponse>('/analytics/feed-forecast', { signal });
}
