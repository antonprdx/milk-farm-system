use serde::Serialize;

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct KpiResponse {
    pub avg_calving_interval_days: Option<f64>,
    pub conception_rate_pct: Option<f64>,
    pub avg_milk_by_lactation: Vec<LactationAvg>,
    pub feed_efficiency: Option<f64>,
    pub avg_days_to_first_ai: Option<f64>,
    pub avg_scc: Option<f64>,
    pub refusal_rate_pct: Option<f64>,
    pub culling_risk: Vec<CullingRiskEntry>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct LactationAvg {
    pub lac: i32,
    pub avg_milk: Option<f64>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct CullingRiskEntry {
    pub animal_id: i32,
    pub name: Option<String>,
    pub life_number: Option<String>,
    pub score: f64,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct Alert {
    pub alert_type: String,
    pub severity: String,
    pub animal_id: Option<i32>,
    pub animal_name: Option<String>,
    pub message: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct AlertsResponse {
    pub alerts: Vec<Alert>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct DailyMilkPoint {
    pub date: String,
    pub total_milk: Option<f64>,
    pub cow_count: Option<i64>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct ForecastPoint {
    pub date: String,
    pub predicted: f64,
    pub lower: f64,
    pub upper: f64,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct BreakPoint {
    pub date: String,
    pub index: i32,
    pub direction: String,
    pub magnitude: f64,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct MilkTrendResponse {
    pub daily: Vec<DailyMilkPoint>,
    pub forecast: Vec<ForecastPoint>,
    pub trend_direction: String,
    pub trend_percent: f64,
    pub confidence: f64,
    pub mape: f64,
    pub model_type: String,
    pub structural_breaks: Vec<BreakPoint>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct ExpectedCalving {
    pub animal_id: i32,
    pub name: Option<String>,
    pub life_number: Option<String>,
    pub insemination_date: Option<String>,
    pub expected_date: String,
    pub days_left: i64,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct ExpectedHeat {
    pub animal_id: i32,
    pub name: Option<String>,
    pub life_number: Option<String>,
    pub last_heat: String,
    pub expected_next: String,
    pub days_until: i64,
    pub overdue: bool,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct DryOffRecommendation {
    pub animal_id: i32,
    pub name: Option<String>,
    pub life_number: Option<String>,
    pub expected_calving: String,
    pub recommended_dry_off: String,
    pub days_until_dry_off: i64,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct ReproductionForecastResponse {
    pub expected_calvings: Vec<ExpectedCalving>,
    pub expected_heats: Vec<ExpectedHeat>,
    pub dry_off_recommendations: Vec<DryOffRecommendation>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct FeedForecastResponse {
    pub weekly_feed_kg: Option<f64>,
    pub predicted_next_week_kg: f64,
    pub avg_per_cow_day_kg: Option<f64>,
    pub milk_per_feed: Option<f64>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct LatestMilkEntry {
    pub animal_id: i32,
    pub name: Option<String>,
    pub date: String,
    pub milk_amount: Option<f64>,
    pub avg_amount: Option<f64>,
    pub isk: Option<f64>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct LactationCurveResponse {
    pub animal_id: i32,
    pub animal_name: Option<String>,
    pub life_number: Option<String>,
    pub lac_number: i32,
    pub calving_date: String,
    pub current_dim: i32,
    pub peak_milk: Option<f64>,
    pub peak_dim: Option<i32>,
    pub predicted_total_305d: Option<f64>,
    pub actual_points: Vec<LactationPoint>,
    pub fitted_curve: Vec<LactationPoint>,
    pub forecast: Vec<LactationPoint>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct LactationPoint {
    pub dim: i32,
    pub milk: Option<f64>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct HealthIndexResponse {
    pub cows: Vec<CowHealthIndex>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct CowHealthIndex {
    pub animal_id: i32,
    pub animal_name: Option<String>,
    pub life_number: Option<String>,
    pub health_score: f64,
    pub milk_deviation_zscore: Option<f64>,
    pub rumination_deviation_zscore: Option<f64>,
    pub activity_deviation_zscore: Option<f64>,
    pub scc_deviation_zscore: Option<f64>,
    pub risk_level: String,
    pub top_concern: Option<String>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct FertilityWindowResponse {
    pub cows: Vec<CowFertilityWindow>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct CowFertilityWindow {
    pub animal_id: i32,
    pub animal_name: Option<String>,
    pub life_number: Option<String>,
    pub days_since_calving: Option<i64>,
    pub activity_signal: Option<f64>,
    pub rumination_signal: Option<f64>,
    pub milk_signal: Option<f64>,
    pub combined_score: f64,
    pub window_status: String,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct ProfitabilityResponse {
    pub cows: Vec<CowProfitability>,
    pub herd_avg_margin_day: Option<f64>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct CowProfitability {
    pub animal_id: i32,
    pub animal_name: Option<String>,
    pub life_number: Option<String>,
    pub avg_daily_milk: Option<f64>,
    pub avg_daily_feed: Option<f64>,
    pub estimated_milk_revenue_day: Option<f64>,
    pub estimated_feed_cost_day: Option<f64>,
    pub estimated_margin_day: Option<f64>,
    pub margin_30d: Option<f64>,
    pub feed_cost_ratio: Option<f64>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct SeasonalResponse {
    pub monthly_indices: Vec<MonthlyIndex>,
    pub trend_7d: Option<f64>,
    pub trend_30d: Option<f64>,
    pub current_seasonal_factor: Option<f64>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct MonthlyIndex {
    pub month: i32,
    pub month_name: String,
    pub avg_daily_milk: Option<f64>,
    pub seasonal_index: Option<f64>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct MastitisRiskResponse {
    pub cows: Vec<MastitisRiskEntry>,
    pub model_version: String,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct MastitisRiskEntry {
    pub animal_id: i32,
    pub animal_name: Option<String>,
    pub life_number: Option<String>,
    pub risk_score: f64,
    pub risk_level: String,
    pub contributing_factors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct CullingSurvivalResponse {
    pub cows: Vec<CullingSurvivalEntry>,
    pub model_version: String,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct CullingSurvivalEntry {
    pub animal_id: i32,
    pub animal_name: Option<String>,
    pub life_number: Option<String>,
    pub expected_days_remaining: Option<i64>,
    pub risk_score: f64,
    pub risk_factors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct EnergyBalanceResponse {
    pub cows: Vec<CowEnergyBalance>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct CowEnergyBalance {
    pub animal_id: i32,
    pub animal_name: Option<String>,
    pub life_number: Option<String>,
    pub avg_fat_pct: Option<f64>,
    pub avg_protein_pct: Option<f64>,
    pub fat_protein_ratio: Option<f64>,
    pub status: String,
    pub trend_7d: Option<f64>,
    pub trend_30d: Option<f64>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct QuarterHealthResponse {
    pub cows: Vec<CowQuarterHealth>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct CowQuarterHealth {
    pub animal_id: i32,
    pub animal_name: Option<String>,
    pub life_number: Option<String>,
    pub lf_conductivity: Option<f64>,
    pub lr_conductivity: Option<f64>,
    pub rf_conductivity: Option<f64>,
    pub rr_conductivity: Option<f64>,
    pub avg_conductivity: Option<f64>,
    pub max_asymmetry: Option<f64>,
    pub worst_quarter: Option<String>,
    pub risk_level: String,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct MilkForecastDay {
    pub day_offset: i32,
    pub predicted_milk: f64,
    pub lower_bound: f64,
    pub upper_bound: f64,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct MilkForecastDataResponse {
    pub animal_id: i32,
    pub animal_name: Option<String>,
    pub current_daily_avg: Option<f64>,
    pub forecast: Vec<MilkForecastDay>,
    pub model_version: String,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct ClusterCowEntry {
    pub animal_id: i32,
    pub animal_name: Option<String>,
    pub cluster_id: i32,
    pub cluster_name: String,
    pub avg_milk: f64,
    pub avg_rumination: f64,
    pub distance_to_center: f64,
    pub model_version: String,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct CowClusterResponse {
    pub clusters: Vec<ClusterCowEntry>,
    pub cluster_names: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct EstrusPrediction {
    pub animal_id: i32,
    pub animal_name: Option<String>,
    pub estrus_probability: f64,
    pub status: String,
    pub contributing_signals: Vec<String>,
    pub optimal_window: Option<String>,
    pub model_version: String,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct EstrusResponse {
    pub predictions: Vec<EstrusPrediction>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct EquipmentAnomalyEntry {
    pub animal_id: i32,
    pub animal_name: Option<String>,
    pub is_anomaly: bool,
    pub anomaly_score: f64,
    pub severity: String,
    pub flags: Vec<String>,
    pub device_address: Option<i32>,
    pub model_version: String,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct EquipmentAnomalyResponse {
    pub entries: Vec<EquipmentAnomalyEntry>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct FeedRecommendationEntry {
    pub animal_id: i32,
    pub animal_name: Option<String>,
    pub current_feed_avg: f64,
    pub recommended_feed: f64,
    pub difference_kg: f64,
    pub suggestion: String,
    pub dim_days: i32,
    pub lactation_number: i32,
    pub model_version: String,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct FeedRecommendationResponse {
    pub recommendations: Vec<FeedRecommendationEntry>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct KetosisWarningEntry {
    pub animal_id: i32,
    pub animal_name: Option<String>,
    pub risk_probability: f64,
    pub risk_type: String,
    pub severity: String,
    pub fpr_current: f64,
    pub fpr_trend: f64,
    pub contributing_factors: Vec<String>,
    pub model_version: String,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct KetosisWarningResponse {
    pub predictions: Vec<KetosisWarningEntry>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct CowFeedEfficiency {
    pub animal_id: i32,
    pub animal_name: Option<String>,
    pub life_number: Option<String>,
    pub avg_daily_milk: Option<f64>,
    pub avg_daily_feed: Option<f64>,
    pub feed_efficiency: Option<f64>,
    pub milk_per_feed: Option<f64>,
    pub efficiency_rank: Option<i32>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct FeedEfficiencyResponse {
    pub cows: Vec<CowFeedEfficiency>,
    pub herd_avg_efficiency: Option<f64>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct DryOffRecommendationEntry {
    pub animal_id: i32,
    pub animal_name: Option<String>,
    pub life_number: Option<String>,
    pub expected_calving_date: Option<String>,
    pub current_daily_milk: Option<f64>,
    pub recommended_dry_off_date: Option<String>,
    pub days_until_dry_off: Option<i64>,
    pub scc_status: String,
    pub readiness: String,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct DryOffOptimizerResponse {
    pub cows: Vec<DryOffRecommendationEntry>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct LifetimeValueEntry {
    pub animal_id: i32,
    pub animal_name: Option<String>,
    pub life_number: Option<String>,
    pub age_years: Option<f64>,
    pub lactation_count: i64,
    pub total_milk_produced: Option<f64>,
    pub estimated_remaining_lactations: i32,
    pub projected_milk_value: Option<f64>,
    pub projected_feed_cost: Option<f64>,
    pub projected_net_value: Option<f64>,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct LifetimeValueResponse {
    pub cows: Vec<LifetimeValueEntry>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct AnimalSummaryResponse {
    pub animal_id: i32,
    pub health_index: Option<CowHealthIndex>,
    pub mastitis_risk: Option<MastitisRiskEntry>,
    pub estrus: Option<EstrusPrediction>,
    pub energy_balance: Option<CowEnergyBalance>,
    pub feed_recommendation: Option<FeedRecommendationEntry>,
    pub ketosis_warning: Option<KetosisWarningEntry>,
    pub lifetime_value: Option<LifetimeValueEntry>,
    pub culling_risk: Option<CullingSurvivalEntry>,
    pub cluster: Option<ClusterCowEntry>,
}
