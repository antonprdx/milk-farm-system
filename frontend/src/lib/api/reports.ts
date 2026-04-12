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

export interface HerdOverviewRow {
	date: string;
	cow_count: number;
	total_milk: number | null;
	avg_day_production: number | null;
	total_milkings: number | null;
	total_refusals: number | null;
	total_failures: number | null;
	milk_separated: number | null;
	avg_scc: number | null;
}

export interface HerdOverviewResponse {
	period: HerdOverviewRow[];
	avg_cow_count: number;
	avg_milk: number | null;
	avg_milkings: number | null;
	avg_failures: number | null;
	avg_scc: number | null;
}

export interface RestFeedRow {
	animal_id: number;
	animal_name: string | null;
	life_number: string | null;
	feed_date: string;
	feed_number: number;
	total_planned: number;
	rest_feed: number | null;
	rest_feed_pct: number | null;
}

export interface RestFeedResponse {
	rows: RestFeedRow[];
	total_rest_feed_pct: number | null;
}

export interface RobotPerformanceRow {
	device_address: number | null;
	date: string;
	avg_milk_speed: number | null;
	max_milk_speed: number | null;
	milkings: number;
	avg_lf_milk_time: number | null;
	avg_lr_milk_time: number | null;
	avg_rf_milk_time: number | null;
	avg_rr_milk_time: number | null;
	avg_lf_dead_milk_time: number | null;
	avg_lr_dead_milk_time: number | null;
	avg_rf_dead_milk_time: number | null;
	avg_rr_dead_milk_time: number | null;
}

export interface FailedMilkingRow {
	animal_id: number;
	animal_name: string | null;
	life_number: string | null;
	visit_datetime: string;
	device_address: number | null;
	milk_yield: number | null;
	lf_colour: string | null;
	lr_colour: string | null;
	rf_colour: string | null;
	rr_colour: string | null;
	lf_conductivity: number | null;
	lr_conductivity: number | null;
	rf_conductivity: number | null;
	rr_conductivity: number | null;
}

export interface UdderHealthRow {
	animal_id: number;
	animal_name: string | null;
	life_number: string | null;
	visit_datetime: string;
	lf_conductivity: number | null;
	lr_conductivity: number | null;
	rf_conductivity: number | null;
	rr_conductivity: number | null;
	lf_colour: string | null;
	lr_colour: string | null;
	rf_colour: string | null;
	rr_colour: string | null;
	latest_scc: number | null;
	milk_yield: number | null;
	deviation_day_prod: number | null;
	attention_quarters: string[];
	separation: string | null;
}

export interface MilkDayProductionTimeRow {
	date: string;
	total_milk: number | null;
	avg_milk_per_cow: number | null;
	cow_count: number;
	milkings: number | null;
	refusals: number | null;
	failures: number | null;
	avg_weight: number | null;
	total_feed: number | null;
	total_rest_feed: number | null;
}

export interface VisitBehaviorRow {
	animal_id: number;
	animal_name: string | null;
	life_number: string | null;
	total_milkings: number;
	total_refusals: number;
	avg_milk_per_milking: number | null;
	avg_duration_seconds: number | null;
	milk_frequency_setting: number | null;
	last_visit: string | null;
}

export interface CalendarCalvingRow {
	animal_id: number;
	animal_name: string | null;
	life_number: string | null;
	lac_number: number | null;
	group_number: number | null;
	last_insemination_date: string | null;
	expected_calving_date: string | null;
	days_until_calving: number | null;
	sire_code: string | null;
	days_pregnant: number | null;
}

export interface CalendarDryOffRow {
	animal_id: number;
	animal_name: string | null;
	life_number: string | null;
	expected_calving_date: string | null;
	recommended_dry_off_date: string | null;
	days_until_dry_off: number | null;
	lac_number: number | null;
}

export interface CalendarHeatRow {
	animal_id: number;
	animal_name: string | null;
	life_number: string | null;
	last_heat_date: string | null;
	expected_heat_date: string | null;
	days_until_heat: number | null;
	days_in_lactation: number | null;
	inseminated: boolean;
	overdue: boolean;
}

export interface CalendarPregnancyCheckRow {
	animal_id: number;
	animal_name: string | null;
	life_number: string | null;
	insemination_date: string | null;
	sire_code: string | null;
	days_since_insemination: number | null;
	pregnancy_confirmed: boolean;
}

export interface CalendarResponse {
	expected_calvings: CalendarCalvingRow[];
	expected_dry_offs: CalendarDryOffRow[];
	expected_heats: CalendarHeatRow[];
	pregnancy_checks: CalendarPregnancyCheckRow[];
}

export interface HealthActivityRow {
	animal_id: number;
	animal_name: string | null;
	life_number: string | null;
	health_index: number | null;
	activity_deviation: number | null;
	rumination_minutes: number | null;
	max_rumination_change_24h: number | null;
	rumination_3day_diff: number | null;
	latest_milk: number | null;
	avg_milk_7d: number | null;
	milk_deviation_pct: number | null;
}

export interface CowRobotEfficiencyRow {
	animal_id: number;
	animal_name: string | null;
	life_number: string | null;
	milk_per_box_time_week: number | null;
	avg_milk_speed: number | null;
	avg_treatment_time: number | null;
	avg_milking_time: number | null;
	milkings_7d: number;
	total_milk_7d: number | null;
	avg_milk_per_milking: number | null;
}

export interface LactationAnalysisPoint {
	dim: number;
	avg_milk: number | null;
	avg_visits: number | null;
	avg_feed: number | null;
	avg_weight: number | null;
	avg_fat: number | null;
	avg_protein: number | null;
	cow_count: number;
}

export interface LactationAnalysisResponse {
	lac_number: number;
	points: LactationAnalysisPoint[];
}

export interface FeedPerTypeDayRow {
	date: string;
	feed_type: string;
	feed_type_name: string;
	total_amount_product: number | null;
	total_amount_dm: number | null;
	total_cost: number | null;
	cost_per_100milk: number | null;
}

export interface FeedPerTypeResponse {
	rows: FeedPerTypeDayRow[];
	avg_cost_per_100milk: number | null;
	total_cost: number | null;
}

export interface FeedPerCowDayRow {
	date: string;
	animal_count: number;
	avg_total_per_cow: number | null;
	avg_concentrate_per_cow: number | null;
	avg_roughage_per_cow: number | null;
	avg_cost_per_cow: number | null;
	avg_rumination_minutes: number | null;
	avg_day_production: number | null;
	avg_lactation_days: number | null;
	feed_efficiency: number | null;
}

export interface HealthTaskRow {
	animal_id: number;
	animal_name: string | null;
	life_number: string | null;
	sick_chance: number;
	sick_chance_status: string;
	milk_drop_kg: number | null;
	conductivity_highest: number | null;
	conductivity_chronic_quarters: string[];
	scc_indication: number | null;
	activity_deviation: number | null;
	rumination_deviation: number | null;
	weight_trend: number | null;
	total_weight_loss: number | null;
	fat_protein_ratio: number | null;
	feed_rest_pct: number | null;
	temperature_highest: number | null;
	colour_attentions: string[];
	milk_trend_deviation: number | null;
	days_in_lactation: number | null;
}

export interface HealthTaskResponse {
	rows: HealthTaskRow[];
}

export interface PregnancyRatePeriod {
	end_date: string;
	eligible: number;
	inseminated: number;
	pregnant: number;
	insemination_rate: number | null;
	conception_rate: number | null;
	pregnancy_rate: number | null;
}

export interface PregnancyRateResponse {
	periods: PregnancyRatePeriod[];
}

export interface TransitionRow {
	animal_id: number;
	animal_name: string | null;
	life_number: string | null;
	days_relative: number;
	milk_24h: number | null;
	sick_chance: number | null;
	rumination_3day_diff: number | null;
	rumination_minutes: number | null;
	feed_total: number | null;
	feed_rest: number | null;
	latest_scc: number | null;
}

export interface TransitionResponse {
	rows: TransitionRow[];
}

export async function getMilkSummary(from?: string, till?: string, signal?: AbortSignal) {
	return api<MilkSummary>(
		`/reports/milk-summary${buildQuery({ from_date: from, till_date: till })}`,
		{ signal },
	);
}

export async function getReproductionSummary(from?: string, till?: string, signal?: AbortSignal) {
	return api<ReproductionSummary>(
		`/reports/reproduction-summary${buildQuery({ from_date: from, till_date: till })}`,
		{ signal },
	);
}

export async function getFeedSummary(from?: string, till?: string, signal?: AbortSignal) {
	return api<FeedSummary>(
		`/reports/feed-summary${buildQuery({ from_date: from, till_date: till })}`,
		{ signal },
	);
}

export async function getHerdOverview(from?: string, till?: string, signal?: AbortSignal) {
	return api<HerdOverviewResponse>(
		`/reports/herd-overview${buildQuery({ from_date: from, till_date: till })}`,
		{ signal },
	);
}

export async function getRestFeed(from?: string, till?: string, signal?: AbortSignal) {
	return api<RestFeedResponse>(
		`/reports/rest-feed${buildQuery({ from_date: from, till_date: till })}`,
		{ signal },
	);
}

export async function getRobotPerformance(from?: string, till?: string, signal?: AbortSignal) {
	return api<RobotPerformanceRow[]>(
		`/reports/robot-performance${buildQuery({ from_date: from, till_date: till })}`,
		{ signal },
	);
}

export async function getFailedMilkings(from?: string, till?: string, signal?: AbortSignal) {
	return api<FailedMilkingRow[]>(
		`/reports/failed-milkings${buildQuery({ from_date: from, till_date: till })}`,
		{ signal },
	);
}

export async function getUdderHealthWorklist(signal?: AbortSignal) {
	return api<{ rows: UdderHealthRow[] }>('/reports/udder-health-worklist', { signal });
}

export async function getUdderHealthAnalyze(signal?: AbortSignal) {
	return api<{ rows: UdderHealthRow[] }>('/reports/udder-health-analyze', { signal });
}

export async function getMilkDayProductionTime(from?: string, till?: string, signal?: AbortSignal) {
	return api<MilkDayProductionTimeRow[]>(
		`/reports/milk-day-production-time${buildQuery({ from_date: from, till_date: till })}`,
		{ signal },
	);
}

export async function getVisitBehavior(from?: string, till?: string, signal?: AbortSignal) {
	return api<VisitBehaviorRow[]>(
		`/reports/visit-behavior${buildQuery({ from_date: from, till_date: till })}`,
		{ signal },
	);
}

export async function getCalendar(signal?: AbortSignal) {
	return api<CalendarResponse>('/reports/calendar', { signal });
}

export async function getHealthActivityRumination(signal?: AbortSignal) {
	return api<HealthActivityRow[]>('/reports/health-activity-rumination', { signal });
}

export async function getCowRobotEfficiency(signal?: AbortSignal) {
	return api<CowRobotEfficiencyRow[]>('/reports/cow-robot-efficiency', { signal });
}

export async function getLactationAnalysis(lacNumber?: number, signal?: AbortSignal) {
	return api<LactationAnalysisResponse[]>(
		`/reports/lactation-analysis${buildQuery({ lac_number: lacNumber })}`,
		{ signal },
	);
}

export async function getFeedPerTypeDay(from?: string, till?: string, signal?: AbortSignal) {
	return api<FeedPerTypeResponse>(
		`/reports/feed-per-type-day${buildQuery({ from_date: from, till_date: till })}`,
		{ signal },
	);
}

export async function getFeedPerCowDay(from?: string, till?: string, signal?: AbortSignal) {
	return api<FeedPerCowDayRow[]>(
		`/reports/feed-per-cow-day${buildQuery({ from_date: from, till_date: till })}`,
		{ signal },
	);
}

export async function getHealthTask(signal?: AbortSignal) {
	return api<HealthTaskResponse>('/reports/health-task', { signal });
}

export async function getPregnancyRate(signal?: AbortSignal) {
	return api<PregnancyRateResponse>('/reports/pregnancy-rate', { signal });
}

export async function getTransitionReport(signal?: AbortSignal) {
	return api<TransitionResponse>('/reports/transition', { signal });
}

export function getExportUrl(
	type: 'milk' | 'reproduction' | 'feed',
	from?: string,
	till?: string,
	format?: 'csv' | 'pdf',
): string {
	const base = import.meta.env.VITE_API_BASE || '/api/v1';
	const suffix = format === 'pdf' ? `/${type}/pdf` : `/${type}`;
	return `${base}/reports/export${suffix}${buildQuery({ from_date: from, till_date: till })}`;
}

export function getReportExportUrl(
	reportType: string,
	from?: string,
	till?: string,
	format: 'csv' | 'pdf' = 'csv',
): string {
	const base = import.meta.env.VITE_API_BASE || '/api/v1';
	const ext = format === 'pdf' ? 'pdf' : 'csv';
	return `${base}/reports/export/${reportType}/${ext}${buildQuery({ from_date: from, till_date: till })}`;
}
