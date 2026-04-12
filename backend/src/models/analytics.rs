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
pub struct MilkTrendResponse {
    pub daily: Vec<DailyMilkPoint>,
    pub forecast: Vec<ForecastPoint>,
    pub trend_direction: String,
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
