use serde::Serialize;

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct HerdOverviewRow {
    pub date: String,
    pub cow_count: i64,
    pub total_milk: Option<f64>,
    pub avg_day_production: Option<f64>,
    pub total_milkings: Option<i64>,
    pub total_refusals: Option<i64>,
    pub total_failures: Option<i64>,
    pub milk_separated: Option<i64>,
    pub avg_scc: Option<f64>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct HerdOverviewResponse {
    pub period: Vec<HerdOverviewRow>,
    pub avg_cow_count: f64,
    pub avg_milk: Option<f64>,
    pub avg_milkings: Option<f64>,
    pub avg_failures: Option<f64>,
    pub avg_scc: Option<f64>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct RestFeedRow {
    pub animal_id: i32,
    pub animal_name: Option<String>,
    pub life_number: Option<String>,
    pub feed_date: String,
    pub feed_number: i32,
    pub total_planned: f64,
    pub rest_feed: Option<i32>,
    pub rest_feed_pct: Option<f64>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct RestFeedResponse {
    pub rows: Vec<RestFeedRow>,
    pub total_rest_feed_pct: Option<f64>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct CowDailyProductionRow {
    pub animal_id: i32,
    pub animal_name: Option<String>,
    pub life_number: Option<String>,
    pub date: String,
    pub milk_amount: Option<f64>,
    pub avg_amount: Option<f64>,
    pub avg_weight: Option<f64>,
    pub isk: Option<f64>,
    pub scc: Option<i32>,
    pub fat_pct: Option<f64>,
    pub protein_pct: Option<f64>,
    pub lactose_pct: Option<f64>,
    pub feed_total: Option<f64>,
    pub feed_rest: Option<i32>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct RobotPerformanceRow {
    pub device_address: Option<i32>,
    pub date: String,
    pub avg_milk_speed: Option<f64>,
    pub max_milk_speed: Option<f64>,
    pub milkings: i64,
    pub avg_lf_milk_time: Option<f64>,
    pub avg_lr_milk_time: Option<f64>,
    pub avg_rf_milk_time: Option<f64>,
    pub avg_rr_milk_time: Option<f64>,
    pub avg_lf_dead_milk_time: Option<f64>,
    pub avg_lr_dead_milk_time: Option<f64>,
    pub avg_rf_dead_milk_time: Option<f64>,
    pub avg_rr_dead_milk_time: Option<f64>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct FailedMilkingRow {
    pub animal_id: i32,
    pub animal_name: Option<String>,
    pub life_number: Option<String>,
    pub visit_datetime: String,
    pub device_address: Option<i32>,
    pub milk_yield: Option<f64>,
    pub lf_colour: Option<String>,
    pub lr_colour: Option<String>,
    pub rf_colour: Option<String>,
    pub rr_colour: Option<String>,
    pub lf_conductivity: Option<i32>,
    pub lr_conductivity: Option<i32>,
    pub rf_conductivity: Option<i32>,
    pub rr_conductivity: Option<i32>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct UdderHealthRow {
    pub animal_id: i32,
    pub animal_name: Option<String>,
    pub life_number: Option<String>,
    pub visit_datetime: String,
    pub lf_conductivity: Option<i32>,
    pub lr_conductivity: Option<i32>,
    pub rf_conductivity: Option<i32>,
    pub rr_conductivity: Option<i32>,
    pub lf_colour: Option<String>,
    pub lr_colour: Option<String>,
    pub rf_colour: Option<String>,
    pub rr_colour: Option<String>,
    pub latest_scc: Option<i32>,
    pub milk_yield: Option<f64>,
    pub deviation_day_prod: Option<f64>,
    pub attention_quarters: Vec<String>,
    pub separation: Option<String>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct UdderHealthResponse {
    pub rows: Vec<UdderHealthRow>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct MilkDayProductionTimeRow {
    pub date: String,
    pub total_milk: Option<f64>,
    pub avg_milk_per_cow: Option<f64>,
    pub cow_count: i64,
    pub milkings: Option<i64>,
    pub refusals: Option<i64>,
    pub failures: Option<i64>,
    pub avg_weight: Option<f64>,
    pub total_feed: Option<f64>,
    pub total_rest_feed: Option<i64>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct VisitBehaviorRow {
    pub animal_id: i32,
    pub animal_name: Option<String>,
    pub life_number: Option<String>,
    pub total_milkings: i64,
    pub total_refusals: i64,
    pub avg_milk_per_milking: Option<f64>,
    pub avg_duration_seconds: Option<f64>,
    pub milk_frequency_setting: Option<i32>,
    pub last_visit: Option<String>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct CalendarCalvingRow {
    pub animal_id: i32,
    pub animal_name: Option<String>,
    pub life_number: Option<String>,
    pub lac_number: Option<i32>,
    pub group_number: Option<i32>,
    pub last_insemination_date: Option<String>,
    pub expected_calving_date: Option<String>,
    pub days_until_calving: Option<i64>,
    pub sire_code: Option<String>,
    pub days_pregnant: Option<i64>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct CalendarDryOffRow {
    pub animal_id: i32,
    pub animal_name: Option<String>,
    pub life_number: Option<String>,
    pub expected_calving_date: Option<String>,
    pub recommended_dry_off_date: Option<String>,
    pub days_until_dry_off: Option<i64>,
    pub lac_number: Option<i32>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct CalendarHeatRow {
    pub animal_id: i32,
    pub animal_name: Option<String>,
    pub life_number: Option<String>,
    pub last_heat_date: Option<String>,
    pub expected_heat_date: Option<String>,
    pub days_until_heat: Option<i64>,
    pub days_in_lactation: Option<i64>,
    pub inseminated: bool,
    pub overdue: bool,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct CalendarPregnancyCheckRow {
    pub animal_id: i32,
    pub animal_name: Option<String>,
    pub life_number: Option<String>,
    pub insemination_date: Option<String>,
    pub sire_code: Option<String>,
    pub days_since_insemination: Option<i64>,
    pub pregnancy_confirmed: bool,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct CalendarResponse {
    pub expected_calvings: Vec<CalendarCalvingRow>,
    pub expected_dry_offs: Vec<CalendarDryOffRow>,
    pub expected_heats: Vec<CalendarHeatRow>,
    pub pregnancy_checks: Vec<CalendarPregnancyCheckRow>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct HealthActivityRow {
    pub animal_id: i32,
    pub animal_name: Option<String>,
    pub life_number: Option<String>,
    pub health_index: Option<f64>,
    pub activity_deviation: Option<f64>,
    pub rumination_minutes: Option<i32>,
    pub max_rumination_change_24h: Option<i32>,
    pub rumination_3day_diff: Option<i32>,
    pub latest_milk: Option<f64>,
    pub avg_milk_7d: Option<f64>,
    pub milk_deviation_pct: Option<f64>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct CowRobotEfficiencyRow {
    pub animal_id: i32,
    pub animal_name: Option<String>,
    pub life_number: Option<String>,
    pub milk_per_box_time_week: Option<f64>,
    pub avg_milk_speed: Option<f64>,
    pub avg_treatment_time: Option<f64>,
    pub avg_milking_time: Option<f64>,
    pub milkings_7d: i64,
    pub total_milk_7d: Option<f64>,
    pub avg_milk_per_milking: Option<f64>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct LactationAnalysisPoint {
    pub dim: i32,
    pub avg_milk: Option<f64>,
    pub avg_visits: Option<f64>,
    pub avg_feed: Option<f64>,
    pub avg_weight: Option<f64>,
    pub avg_fat: Option<f64>,
    pub avg_protein: Option<f64>,
    pub cow_count: i64,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct LactationAnalysisResponse {
    pub lac_number: i32,
    pub points: Vec<LactationAnalysisPoint>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct FeedPerTypeDayRow {
    pub date: String,
    pub feed_type: String,
    pub feed_type_name: String,
    pub total_amount_product: Option<f64>,
    pub total_amount_dm: Option<f64>,
    pub total_cost: Option<f64>,
    pub cost_per_100milk: Option<f64>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct FeedPerTypeResponse {
    pub rows: Vec<FeedPerTypeDayRow>,
    pub avg_cost_per_100milk: Option<f64>,
    pub total_cost: Option<f64>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct FeedPerCowDayRow {
    pub date: String,
    pub animal_count: i64,
    pub avg_total_per_cow: Option<f64>,
    pub avg_concentrate_per_cow: Option<f64>,
    pub avg_roughage_per_cow: Option<f64>,
    pub avg_cost_per_cow: Option<f64>,
    pub avg_rumination_minutes: Option<f64>,
    pub avg_day_production: Option<f64>,
    pub avg_lactation_days: Option<f64>,
    pub feed_efficiency: Option<f64>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct HealthTaskRow {
    pub animal_id: i32,
    pub animal_name: Option<String>,
    pub life_number: Option<String>,
    pub sick_chance: f64,
    pub sick_chance_status: String,
    pub milk_drop_kg: Option<f64>,
    pub conductivity_highest: Option<i32>,
    pub conductivity_chronic_quarters: Vec<String>,
    pub scc_indication: Option<i32>,
    pub activity_deviation: Option<f64>,
    pub rumination_deviation: Option<i32>,
    pub weight_trend: Option<f64>,
    pub total_weight_loss: Option<f64>,
    pub fat_protein_ratio: Option<f64>,
    pub feed_rest_pct: Option<f64>,
    pub temperature_highest: Option<f64>,
    pub colour_attentions: Vec<String>,
    pub milk_trend_deviation: Option<f64>,
    pub days_in_lactation: Option<i64>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct HealthTaskResponse {
    pub rows: Vec<HealthTaskRow>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct PregnancyRatePeriod {
    pub end_date: String,
    pub eligible: i64,
    pub inseminated: i64,
    pub pregnant: i64,
    pub insemination_rate: Option<f64>,
    pub conception_rate: Option<f64>,
    pub pregnancy_rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct PregnancyRateResponse {
    pub periods: Vec<PregnancyRatePeriod>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct TransitionRow {
    pub animal_id: i32,
    pub animal_name: Option<String>,
    pub life_number: Option<String>,
    pub days_relative: i64,
    pub milk_24h: Option<f64>,
    pub sick_chance: Option<f64>,
    pub rumination_3day_diff: Option<i32>,
    pub rumination_minutes: Option<i32>,
    pub feed_total: Option<f64>,
    pub feed_rest: Option<i32>,
    pub latest_scc: Option<i32>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct TransitionResponse {
    pub rows: Vec<TransitionRow>,
}
