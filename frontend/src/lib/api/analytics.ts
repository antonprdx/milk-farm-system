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

export interface LatestMilkEntry {
	animal_id: number;
	name: string | null;
	date: string;
	milk_amount: number | null;
	avg_amount: number | null;
	isk: number | null;
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

export function getLatestMilk(signal?: AbortSignal) {
	return api<LatestMilkEntry[]>('/analytics/latest-milk', { signal });
}

export interface LactationPoint {
	dim: number;
	milk: number | null;
}

export interface LactationCurveResponse {
	animal_id: number;
	animal_name: string | null;
	life_number: string | null;
	lac_number: number;
	calving_date: string;
	current_dim: number;
	peak_milk: number | null;
	peak_dim: number | null;
	predicted_total_305d: number | null;
	actual_points: LactationPoint[];
	fitted_curve: LactationPoint[];
	forecast: LactationPoint[];
}

export function getLactationCurves(animalId?: number, signal?: AbortSignal) {
	const params: Record<string, string> = {};
	if (animalId) params.animal_id = String(animalId);
	return api<LactationCurveResponse[]>(
		`/analytics/lactation-curves${buildQuery(params)}`,
		{ signal },
	);
}

export interface CowHealthIndex {
	animal_id: number;
	animal_name: string | null;
	life_number: string | null;
	health_score: number;
	milk_deviation_zscore: number | null;
	rumination_deviation_zscore: number | null;
	activity_deviation_zscore: number | null;
	scc_deviation_zscore: number | null;
	risk_level: string;
	top_concern: string | null;
}

export interface HealthIndexResponse {
	cows: CowHealthIndex[];
}

export function getHealthIndex(signal?: AbortSignal) {
	return api<HealthIndexResponse>('/analytics/health-index', { signal });
}

export interface CowFertilityWindow {
	animal_id: number;
	animal_name: string | null;
	life_number: string | null;
	days_since_calving: number | null;
	activity_signal: number | null;
	rumination_signal: number | null;
	milk_signal: number | null;
	combined_score: number;
	window_status: string;
}

export interface FertilityWindowResponse {
	cows: CowFertilityWindow[];
}

export function getFertilityWindow(signal?: AbortSignal) {
	return api<FertilityWindowResponse>('/analytics/fertility-window', { signal });
}

export interface CowProfitability {
	animal_id: number;
	animal_name: string | null;
	life_number: string | null;
	avg_daily_milk: number | null;
	avg_daily_feed: number | null;
	estimated_milk_revenue_day: number | null;
	estimated_feed_cost_day: number | null;
	estimated_margin_day: number | null;
	margin_30d: number | null;
	feed_cost_ratio: number | null;
}

export interface ProfitabilityResponse {
	cows: CowProfitability[];
	herd_avg_margin_day: number | null;
}

export function getProfitability(
	milkPrice?: number,
	feedPrice?: number,
	signal?: AbortSignal,
) {
	const params: Record<string, string> = {};
	if (milkPrice) params.milk_price = String(milkPrice);
	if (feedPrice) params.feed_price = String(feedPrice);
	return api<ProfitabilityResponse>(
		`/analytics/profitability${buildQuery(params)}`,
		{ signal },
	);
}

export interface MonthlyIndex {
	month: number;
	month_name: string;
	avg_daily_milk: number | null;
	seasonal_index: number | null;
}

export interface SeasonalResponse {
	monthly_indices: MonthlyIndex[];
	trend_7d: number | null;
	trend_30d: number | null;
	current_seasonal_factor: number | null;
}

export function getSeasonal(signal?: AbortSignal) {
	return api<SeasonalResponse>('/analytics/seasonal', { signal });
}

export interface MastitisRiskEntry {
	animal_id: number;
	animal_name: string | null;
	life_number: string | null;
	risk_score: number;
	risk_level: string;
	contributing_factors: string[];
}

export interface MastitisRiskResponse {
	cows: MastitisRiskEntry[];
	model_version: string;
}

export function getMastitisRisk(signal?: AbortSignal) {
	return api<MastitisRiskResponse>('/analytics/mastitis-risk', { signal });
}

export interface CullingSurvivalEntry {
	animal_id: number;
	animal_name: string | null;
	life_number: string | null;
	expected_days_remaining: number | null;
	risk_score: number;
	risk_factors: string[];
}

export interface CullingSurvivalResponse {
	cows: CullingSurvivalEntry[];
	model_version: string;
}

export function getCullingSurvival(signal?: AbortSignal) {
	return api<CullingSurvivalResponse>('/analytics/culling-survival', { signal });
}

export interface CowEnergyBalance {
	animal_id: number;
	animal_name: string | null;
	life_number: string | null;
	avg_fat_pct: number | null;
	avg_protein_pct: number | null;
	fat_protein_ratio: number | null;
	status: string;
	trend_7d: number | null;
	trend_30d: number | null;
}

export interface EnergyBalanceResponse {
	cows: CowEnergyBalance[];
}

export function getEnergyBalance(signal?: AbortSignal) {
	return api<EnergyBalanceResponse>('/analytics/energy-balance', { signal });
}

export interface CowQuarterHealth {
	animal_id: number;
	animal_name: string | null;
	life_number: string | null;
	lf_conductivity: number | null;
	lr_conductivity: number | null;
	rf_conductivity: number | null;
	rr_conductivity: number | null;
	avg_conductivity: number | null;
	max_asymmetry: number | null;
	worst_quarter: string | null;
	risk_level: string;
}

export interface QuarterHealthResponse {
	cows: CowQuarterHealth[];
}

export function getQuarterHealth(signal?: AbortSignal) {
	return api<QuarterHealthResponse>('/analytics/quarter-health', { signal });
}

export interface MilkForecastDay {
	day_offset: number;
	predicted_milk: number;
	lower_bound: number;
	upper_bound: number;
}

export interface MilkForecastResponse {
	animal_id: number;
	animal_name: string | null;
	current_daily_avg: number | null;
	forecast: MilkForecastDay[];
	model_version: string;
}

export function getMilkForecast(animalId: number, days?: number, signal?: AbortSignal) {
	const params: Record<string, string> = { animal_id: String(animalId) };
	if (days) params.days = String(days);
	return api<MilkForecastResponse>(
		`/analytics/milk-forecast${buildQuery(params)}`,
		{ signal },
	);
}

export interface ClusterEntry {
	animal_id: number;
	animal_name: string | null;
	cluster_id: number;
	cluster_name: string;
	avg_milk: number;
	avg_rumination: number;
	distance_to_center: number;
	model_version: string;
}

export interface ClusterResponse {
	clusters: ClusterEntry[];
	cluster_names: Record<string, string>;
}

export function getCowClusters(days?: number, signal?: AbortSignal) {
	const params: Record<string, string> = {};
	if (days) params.days = String(days);
	return api<ClusterResponse>(
		`/analytics/cow-clusters${buildQuery(params)}`,
		{ signal },
	);
}
