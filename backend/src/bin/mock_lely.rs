use std::sync::Arc;

use axum::Router;
use axum::extract::{Query, State};
use axum::response::Json;
use axum::routing::{get, post};
use chrono::{Datelike, Duration, NaiveDate, TimeZone, Utc};
use rand::Rng;
use rand::SeedableRng;
use rand::distr::Distribution;
use rand::seq::IndexedRandom;
use serde::{Deserialize, Serialize};
use serde_json::Value;

struct Normal {
    mean: f64,
    stddev: f64,
}

impl Normal {
    fn new(mean: f64, stddev: f64) -> Self {
        Self { mean, stddev }
    }
}

impl Distribution<f64> for Normal {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> f64 {
        let u1: f64 = rng.random();
        let u2: f64 = rng.random();
        let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
        self.mean + self.stddev * z
    }
}

fn normal_range(rng: &mut impl Rng, mean: f64, stddev: f64, min: f64, max: f64) -> f64 {
    Normal::new(mean, stddev).sample(rng).clamp(min, max)
}

fn pick<'a, T>(rng: &mut impl Rng, slice: &'a [T]) -> &'a T {
    slice.choose(rng).unwrap()
}

fn fmt_life_number(rng: &mut impl Rng, idx: usize) -> String {
    format!("RU{:03}{:012}", rng.random_range(0u32..999), idx)
}

fn fmt_date(dt: chrono::DateTime<Utc>) -> String {
    dt.format("%Y-%m-%dT%H:%M:%S.000Z").to_string()
}

fn fmt_date_only(d: NaiveDate) -> String {
    d.format("%Y-%m-%dT00:00:00.000Z").to_string()
}

fn wood_milk(day_in_lactation: i64, lac_number: i32, rng: &mut impl Rng) -> f64 {
    let b = 0.2_f64;
    let c = 0.0035_f64;
    let peak = match lac_number {
        1 => normal_range(rng, 25.0, 3.0, 18.0, 32.0),
        2 => normal_range(rng, 30.0, 3.5, 22.0, 38.0),
        _ => normal_range(rng, 33.0, 4.0, 24.0, 42.0),
    };
    let t = day_in_lactation as f64;
    if t < 1.0 {
        return 0.0;
    }
    let a = peak / (t.powf(b) * (-c * t).exp());
    let base = a * t.powf(b) * (-c * t).exp();
    let noise = normal_range(rng, 1.0, 0.08, 0.7, 1.3);
    (base * noise).max(0.0).round() / 10.0
}

const COW_NAMES: &[&str] = &[
    "Милка",
    "Зорька",
    "Бурёнка",
    "Красавица",
    "Рябушка",
    "Дайка",
    "Пеструха",
    "Машка",
    "Дашка",
    "Ночка",
    "Звёздочка",
    "Весна",
    "Дымка",
    "Карамелька",
    "Булочка",
    "Ласка",
    "Печенька",
    "Снежка",
    "Утренняя",
    "Крошка",
    "Маргаритка",
    "Ромашка",
    "Ириска",
    "Кнопка",
    "Муся",
];

const HAIR_COLORS: &[&str] = &["RH", "WH", "RD", "BN", "BL"];

const SIRE_CODES: &[&str] = &[
    "SIRE001", "SIRE002", "SIRE003", "SIRE004", "SIRE005", "SIRE006", "SIRE007", "SIRE008",
    "SIRE009", "SIRE010",
];

#[expect(dead_code)]
struct MockState {
    animals: Vec<AnimalData>,
    start_date: NaiveDate,
    end_date: NaiveDate,
}

#[derive(Clone)]
struct AnimalData {
    life_number: String,
    name: String,
    user_number: i64,
    birth_date: NaiveDate,
    hair_color_code: String,
    father_life_number: String,
    mother_life_number: String,
    ucn_number: String,
    responder_number: String,
    gestation: i32,
    calvings: Vec<CalvingData>,
}

#[derive(Clone)]
struct CalvingData {
    date: NaiveDate,
    lac_number: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct BaseResponse<T: Serialize> {
    data: T,
    meta: Meta,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct Meta {
    code: i32,
    message: Option<String>,
    errors: Option<Vec<Value>>,
    detail: Option<String>,
}

fn ok_response<T: Serialize>(data: T) -> Json<BaseResponse<T>> {
    Json(BaseResponse {
        data,
        meta: Meta {
            code: 200,
            message: None,
            errors: None,
            detail: None,
        },
    })
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
#[expect(dead_code)]
struct AuthRequest {
    user_name: String,
    password: String,
}

#[derive(Deserialize)]
struct DateRangeParams {
    from_date: Option<String>,
    till_date: Option<String>,
    from_production_date: Option<String>,
    till_production_date: Option<String>,
    from_feed_date: Option<String>,
    till_feed_date: Option<String>,
    from_activity_date: Option<String>,
    till_activity_date: Option<String>,
    from_transfer_date: Option<String>,
    till_transfer_date: Option<String>,
    from_insemination_date: Option<String>,
    till_insemination_date: Option<String>,
    life_number: Option<String>,
}

fn parse_param_date(s: &Option<String>) -> Option<NaiveDate> {
    s.as_ref().and_then(|d| {
        chrono::DateTime::parse_from_rfc3339(d)
            .map(|dt| dt.date_naive())
            .or_else(|_| NaiveDate::parse_from_str(d, "%Y-%m-%d"))
            .ok()
    })
}

fn get_date_range(params: &DateRangeParams) -> (NaiveDate, NaiveDate) {
    let from = parse_param_date(&params.from_date)
        .or_else(|| parse_param_date(&params.from_production_date))
        .or_else(|| parse_param_date(&params.from_feed_date))
        .or_else(|| parse_param_date(&params.from_activity_date))
        .or_else(|| parse_param_date(&params.from_transfer_date))
        .or_else(|| parse_param_date(&params.from_insemination_date));

    let till = parse_param_date(&params.till_date)
        .or_else(|| parse_param_date(&params.till_production_date))
        .or_else(|| parse_param_date(&params.till_feed_date))
        .or_else(|| parse_param_date(&params.till_activity_date))
        .or_else(|| parse_param_date(&params.till_transfer_date))
        .or_else(|| parse_param_date(&params.till_insemination_date));

    (
        from.unwrap_or(Utc::now().date_naive() - Duration::days(30)),
        till.unwrap_or(Utc::now().date_naive()),
    )
}

fn generate_animals(count: usize) -> Vec<AnimalData> {
    let mut rng = rand::rngs::SmallRng::from_os_rng();
    let mut animals = Vec::with_capacity(count);

    let base_date = Utc::now().date_naive();

    for i in 0..count {
        let age_days: i32 = rng.random_range(730..3650);
        let birth_date = base_date - Duration::days(age_days as i64);
        let life_number = fmt_life_number(&mut rng, i);

        let max_calvings = ((age_days - 700) / 400).clamp(0, 8);
        let num_calvings: i32 = if max_calvings > 0 {
            rng.random_range(1..=max_calvings)
        } else {
            0
        };

        let mut calvings = Vec::new();
        for lac in 1..=num_calvings {
            let calving_age: i32 = 700 + (lac - 1) * 400 + rng.random_range(-20..20);
            if calving_age < age_days {
                let calving_date = birth_date + Duration::days(calving_age as i64);
                if calving_date <= base_date {
                    calvings.push(CalvingData {
                        date: calving_date,
                        lac_number: lac,
                    });
                }
            }
        }

        let name_idx = i % COW_NAMES.len();
        animals.push(AnimalData {
            life_number,
            name: format!("{} #{}", COW_NAMES[name_idx], i + 1),
            user_number: (i + 1) as i64,
            birth_date,
            hair_color_code: pick(&mut rng, HAIR_COLORS).to_string(),
            father_life_number: fmt_life_number(&mut rng, count + i * 2),
            mother_life_number: fmt_life_number(&mut rng, count + i * 2 + 1),
            ucn_number: format!("UCN{:06}", rng.random_range(1u32..999999)),
            responder_number: format!("{:015}", rng.random_range(1u64..999999999999999)),
            gestation: if rng.random_bool(0.6) {
                rng.random_range(0..=280)
            } else {
                0
            },
            calvings,
        });
    }

    animals
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct AnimalResponse {
    ani_id: i32,
    life_number: String,
    name: String,
    user_number: i64,
    gender: String,
    birth_date: String,
    hair_color_code: String,
    father_life_number: String,
    mother_life_number: String,
    description: Option<String>,
    ucn_number: String,
    use_as_sire: Option<bool>,
    location: Option<String>,
    group_number: Option<i32>,
    keep: Option<bool>,
    gestation: Option<i32>,
    responder_number: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct DayProduction {
    life_number: String,
    date: String,
    milk_day_production: Option<f64>,
    milk_day_production_average: Option<f64>,
    average_weight: Option<f64>,
    isk: Option<f64>,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct MilkVisitResp {
    life_number: String,
    milking_date: String,
    milking_start_date: Option<String>,
    milk_yield: Option<f64>,
    bottle_number: Option<i32>,
    success_milking: Option<bool>,
    weight: Option<i32>,
    device_address: Option<i32>,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct MilkVisitQualityResp {
    life_number: String,
    milking_date: String,
    device_address: Option<i32>,
    milking_start_date: Option<String>,
    success_milking: Option<bool>,
    milk_yield: Option<f64>,
    bottle_number: Option<i32>,
    milk_temperature: Option<f64>,
    weight: Option<i32>,
    lf_colour_code: Option<String>,
    lr_colour_code: Option<String>,
    rf_colour_code: Option<String>,
    rr_colour_code: Option<String>,
    lf_conductivity: Option<i32>,
    lr_conductivity: Option<i32>,
    rf_conductivity: Option<i32>,
    rr_conductivity: Option<i32>,
    milk_destination: Option<i32>,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct DayProductionQuality {
    life_number: String,
    date: String,
    milk_day_production: Option<f64>,
    milk_day_production_average: Option<f64>,
    average_weight: Option<f64>,
    isk: Option<f64>,
    fat_percentage: Option<f64>,
    protein_percentage: Option<f64>,
    lactose_percentage: Option<f64>,
    scc: Option<i32>,
    mdp_milkings: Option<i32>,
    mdp_refusals: Option<i32>,
    mdp_failures: Option<i32>,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct RobotDataResp {
    device_address: Option<i32>,
    life_number: Option<String>,
    milking_date: String,
    milk_speed: Option<f64>,
    milk_speed_max: Option<f64>,
    lf_milk_time: Option<i32>,
    lr_milk_time: Option<i32>,
    rf_milk_time: Option<i32>,
    rr_milk_time: Option<i32>,
    lf_dead_milk_time: Option<i32>,
    lr_dead_milk_time: Option<i32>,
    rf_dead_milk_time: Option<i32>,
    rr_dead_milk_time: Option<i32>,
    lf_x_position: Option<i32>,
    lf_y_position: Option<i32>,
    lfz_position: Option<i32>,
    lr_x_position: Option<i32>,
    lr_y_position: Option<i32>,
    lr_z_position: Option<i32>,
    rf_x_position: Option<i32>,
    rf_y_position: Option<i32>,
    rf_z_position: Option<i32>,
    rr_x_position: Option<i32>,
    rr_y_position: Option<i32>,
    rr_z_position: Option<i32>,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct FeedDayAmountResp {
    life_number: String,
    feed_date: String,
    feed_number: Option<i32>,
    total: Option<f64>,
    rest_feed: Option<i32>,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct FeedVisitResp {
    device_address: Option<i32>,
    life_number: String,
    feed_date: String,
    feed_name: Option<String>,
    number_of_feed_type: Option<i32>,
    credit: Option<i32>,
    intake: Option<i32>,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct ActivityResp {
    life_number: String,
    activity_date_time: String,
    activity_counter: Option<i32>,
    heat_attention: Option<bool>,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct RuminationResp {
    life_number: String,
    date_time: String,
    eating_seconds: Option<i32>,
    rumination_minutes: Option<i32>,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct GrazingResp {
    date: String,
    total_milking_cows: i32,
    cows14_dil: i32,
    percentage_in_pasture: f64,
    grazing_day_yes_no: bool,
    grazing_time: i32,
    sd_time_pasture: i32,
    cum_pasture_days: i32,
    cum_total_pasturetime: i32,
    tank_number: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct CalvingResp {
    life_number: String,
    calving_date: String,
    remarks: Option<String>,
    lac_number: Option<i32>,
    calves: Option<Vec<CalfResp>>,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct CalfResp {
    gender: String,
    birth_remark: Option<String>,
    keep: Option<bool>,
    weight: Option<f64>,
    born_dead: Option<bool>,
    calf_name: Option<String>,
    hair_color_code: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct InseminationResp {
    life_number: String,
    insemination_date: String,
    insemination_type: String,
    sire_code: Option<String>,
    charge_number: Option<String>,
    insemination_number: Option<i32>,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct PregnancyResp {
    life_number: String,
    pregnancy_date: String,
    pregnancy_type: String,
    insemination_date: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct HeatResp {
    life_number: String,
    heat_date: String,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct DryOffResp {
    life_number: String,
    dry_off_date: String,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct SireResp {
    sire_code: String,
    sire_name: Option<String>,
    life_number: String,
    sire_active: Option<bool>,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct TransferResp {
    life_number: String,
    transfer_date: String,
    transfer_type: String,
    reason_id: Option<i32>,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct BloodLineResp {
    life_number: String,
    blood_type_code: String,
    percentage: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct LocationResp {
    location_id: i32,
    location_name: Option<String>,
    location_description: Option<String>,
    barn_id: Option<i32>,
    feed_group_id: Option<i32>,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct ContactResp {
    contact_id: i32,
    name: Option<String>,
    contact_type_id: i32,
    contact_type_name: Option<String>,
    farm_number: Option<String>,
    phone_cell: Option<String>,
    email: Option<String>,
    active: Option<bool>,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct FeedTypeResp {
    feed_id: i32,
    number_of_feed_type: i32,
    feed_type: String,
    feed_name: String,
    feed_dry_matter_percentage: f64,
    feed_price: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct FeedGroupResp {
    feed_group_id: Option<String>,
    feed_group_name: Option<String>,
    max_milk_yield: f64,
    min_milk_yield: f64,
    average_milk_yield: f64,
    number_of_cows: i32,
}

type AppState = Arc<MockState>;

fn make_router(state: Arc<MockState>) -> Router {
    Router::new()
        .route("/api/authentication/token", post(auth_token))
        .route("/api/animals", get(get_animals))
        .route("/api/milkdayproductions", get(get_milk_day_productions))
        .route("/api/milkvisits", get(get_milk_visits))
        .route("/api/milkvisitsquality", get(get_milk_visit_quality))
        .route("/api/milkdayproductionsquality", get(get_milk_day_quality))
        .route("/api/milkvisitrobotdata", get(get_robot_data))
        .route("/api/feedamounts", get(get_feed_amounts))
        .route("/api/feedvisits", get(get_feed_visits))
        .route("/api/activities", get(get_activities))
        .route("/api/ruminations", get(get_ruminations))
        .route("/api/grazingdatas", get(get_grazing))
        .route("/api/calvings", get(get_calvings))
        .route("/api/inseminations", get(get_inseminations))
        .route("/api/pregnancies", get(get_pregnancies))
        .route("/api/heats", get(get_heats))
        .route("/api/dryoffs", get(get_dry_offs))
        .route("/api/sires", get(get_sires))
        .route("/api/transfers", get(get_transfers))
        .route("/api/bloodlines", get(get_bloodlines))
        .route("/api/contacts", get(get_contacts))
        .route("/api/locations", get(get_locations))
        .route("/api/feedtypes", get(get_feed_types))
        .route("/api/feedgroups", get(get_feed_groups))
        .with_state(state)
}

async fn auth_token(Json(_body): Json<AuthRequest>) -> Json<BaseResponse<String>> {
    ok_response("mock-lely-token-12345".to_string())
}

async fn get_animals(State(st): State<AppState>) -> Json<BaseResponse<Vec<AnimalResponse>>> {
    let mut rng = rand::rngs::SmallRng::from_os_rng();
    let data: Vec<AnimalResponse> = st
        .animals
        .iter()
        .enumerate()
        .map(|(i, a)| AnimalResponse {
            ani_id: (i + 1) as i32,
            life_number: a.life_number.clone(),
            name: a.name.clone(),
            user_number: a.user_number,
            gender: "Female".to_string(),
            birth_date: fmt_date_only(a.birth_date),
            hair_color_code: a.hair_color_code.clone(),
            father_life_number: a.father_life_number.clone(),
            mother_life_number: a.mother_life_number.clone(),
            description: None,
            ucn_number: a.ucn_number.clone(),
            use_as_sire: Some(false),
            location: Some("Корівник 1".to_string()),
            group_number: Some(rng.random_range(1..=5)),
            keep: Some(true),
            gestation: Some(a.gestation),
            responder_number: Some(a.responder_number.clone()),
        })
        .collect();
    ok_response(data)
}

fn filter_animals<'a>(st: &'a MockState, params: &DateRangeParams) -> Vec<&'a AnimalData> {
    if let Some(ref ln) = params.life_number {
        st.animals.iter().filter(|a| a.life_number == *ln).collect()
    } else {
        st.animals.iter().collect()
    }
}

async fn get_milk_day_productions(
    State(st): State<AppState>,
    Query(params): Query<DateRangeParams>,
) -> Json<BaseResponse<Vec<DayProduction>>> {
    let mut rng = rand::rngs::SmallRng::from_os_rng();
    let (from, till) = get_date_range(&params);
    let animals = filter_animals(&st, &params);

    let mut result = Vec::new();
    let mut d = from;
    while d <= till {
        for a in &animals {
            let lac = a.calvings.last().map(|c| c.lac_number).unwrap_or(0);
            if lac == 0 {
                continue;
            }
            let days_since_calving = (d - a.calvings.last().unwrap().date).num_days();
            if !(0..=350).contains(&days_since_calving) {
                continue;
            }
            let milk = wood_milk(days_since_calving, lac, &mut rng);
            if milk > 0.5 {
                result.push(DayProduction {
                    life_number: a.life_number.clone(),
                    date: fmt_date_only(d),
                    milk_day_production: Some((milk * 10.0).round() / 10.0),
                    milk_day_production_average: Some(
                        (milk * normal_range(&mut rng, 1.0, 0.05, 0.8, 1.2) * 10.0).round() / 10.0,
                    ),
                    average_weight: Some(normal_range(&mut rng, 550.0, 50.0, 400.0, 750.0)),
                    isk: Some(normal_range(&mut rng, 85.0, 10.0, 60.0, 110.0)),
                });
            }
        }
        d += Duration::days(1);
    }
    ok_response(result)
}

async fn get_milk_visits(
    State(st): State<AppState>,
    Query(params): Query<DateRangeParams>,
) -> Json<BaseResponse<Vec<MilkVisitResp>>> {
    let mut rng = rand::rngs::SmallRng::from_os_rng();
    let (from, till) = get_date_range(&params);
    let animals = filter_animals(&st, &params);

    let mut result = Vec::new();
    let mut d = from;
    while d <= till {
        for a in &animals {
            let lac = a.calvings.last().map(|c| c.lac_number).unwrap_or(0);
            if lac == 0 {
                continue;
            }
            let days_since_calving = (d - a.calvings.last().unwrap().date).num_days();
            let milk = wood_milk(days_since_calving, lac, &mut rng);
            if milk < 0.5 {
                continue;
            }

            let num_visits = rng.random_range(2..=3);
            for v in 0..num_visits {
                let hour = 5 + v * 8 + rng.random_range(0..3);
                let dt = Utc
                    .with_ymd_and_hms(
                        d.year(),
                        d.month(),
                        d.day(),
                        hour as u32,
                        rng.random_range(0..59),
                        rng.random_range(0..59),
                    )
                    .single()
                    .unwrap_or_default();
                let visit_milk = milk / num_visits as f64;
                let start_dt = dt - Duration::try_minutes(2 + rng.random_range(0..4)).unwrap();

                result.push(MilkVisitResp {
                    life_number: a.life_number.clone(),
                    milking_date: fmt_date(dt),
                    milking_start_date: Some(fmt_date(start_dt)),
                    milk_yield: Some((visit_milk * 10.0).round() / 10.0),
                    bottle_number: Some(rng.random_range(1..=4)),
                    success_milking: Some(rng.random_bool(0.97)),
                    weight: Some(normal_range(&mut rng, 550.0, 30.0, 420.0, 720.0) as i32),
                    device_address: Some(rng.random_range(1..=10)),
                });
            }
        }
        d += Duration::days(1);
    }
    ok_response(result)
}

async fn get_milk_visit_quality(
    State(st): State<AppState>,
    Query(params): Query<DateRangeParams>,
) -> Json<BaseResponse<Vec<MilkVisitQualityResp>>> {
    let mut rng = rand::rngs::SmallRng::from_os_rng();
    let (from, till) = get_date_range(&params);
    let animals = filter_animals(&st, &params);

    let mut result = Vec::new();
    let mut d = from;
    while d <= till {
        for a in &animals {
            let lac = a.calvings.last().map(|c| c.lac_number).unwrap_or(0);
            if lac == 0 {
                continue;
            }
            let days_since_calving = (d - a.calvings.last().unwrap().date).num_days();
            let milk = wood_milk(days_since_calving, lac, &mut rng);
            if milk < 0.5 {
                continue;
            }

            let hour = 5 + rng.random_range(0..12);
            let dt = Utc
                .with_ymd_and_hms(
                    d.year(),
                    d.month(),
                    d.day(),
                    hour as u32,
                    rng.random_range(0..59),
                    0,
                )
                .single()
                .unwrap_or_default();

            result.push(MilkVisitQualityResp {
                life_number: a.life_number.clone(),
                milking_date: fmt_date(dt),
                device_address: Some(rng.random_range(1..=10)),
                milking_start_date: Some(fmt_date(dt - Duration::try_minutes(3).unwrap())),
                success_milking: Some(rng.random_bool(0.97)),
                milk_yield: Some(milk / 2.5),
                bottle_number: Some(rng.random_range(1..=4)),
                milk_temperature: Some(normal_range(&mut rng, 37.5, 1.0, 35.0, 40.0)),
                weight: Some(normal_range(&mut rng, 550.0, 30.0, 420.0, 720.0) as i32),
                lf_colour_code: Some("G".to_string()),
                lr_colour_code: Some("G".to_string()),
                rf_colour_code: Some("G".to_string()),
                rr_colour_code: Some("G".to_string()),
                lf_conductivity: Some(normal_range(&mut rng, 8.0, 2.0, 4.0, 14.0) as i32),
                lr_conductivity: Some(normal_range(&mut rng, 8.0, 2.0, 4.0, 14.0) as i32),
                rf_conductivity: Some(normal_range(&mut rng, 8.0, 2.0, 4.0, 14.0) as i32),
                rr_conductivity: Some(normal_range(&mut rng, 8.0, 2.0, 4.0, 14.0) as i32),
                milk_destination: Some(if rng.random_bool(0.9) { 0 } else { 4 }),
            });
        }
        d += Duration::days(1);
    }
    ok_response(result)
}

async fn get_milk_day_quality(
    State(st): State<AppState>,
    Query(params): Query<DateRangeParams>,
) -> Json<BaseResponse<Vec<DayProductionQuality>>> {
    let mut rng = rand::rngs::SmallRng::from_os_rng();
    let (from, till) = get_date_range(&params);
    let animals = filter_animals(&st, &params);

    let mut result = Vec::new();
    let mut d = from;
    while d <= till {
        for a in &animals {
            let lac = a.calvings.last().map(|c| c.lac_number).unwrap_or(0);
            if lac == 0 {
                continue;
            }
            let days_since_calving = (d - a.calvings.last().unwrap().date).num_days();
            let milk = wood_milk(days_since_calving, lac, &mut rng);
            if milk < 0.5 {
                continue;
            }

            result.push(DayProductionQuality {
                life_number: a.life_number.clone(),
                date: fmt_date_only(d),
                milk_day_production: Some(milk),
                milk_day_production_average: Some(
                    milk * normal_range(&mut rng, 1.0, 0.05, 0.8, 1.2),
                ),
                average_weight: Some(normal_range(&mut rng, 550.0, 50.0, 400.0, 750.0)),
                isk: Some(normal_range(&mut rng, 85.0, 10.0, 60.0, 110.0)),
                fat_percentage: Some(normal_range(&mut rng, 3.8, 0.5, 2.5, 5.5)),
                protein_percentage: Some(normal_range(&mut rng, 3.2, 0.3, 2.5, 4.2)),
                lactose_percentage: Some(normal_range(&mut rng, 4.6, 0.2, 3.8, 5.2)),
                scc: Some(normal_range(&mut rng, 150.0, 100.0, 20.0, 800.0) as i32),
                mdp_milkings: Some(rng.random_range(2..=3)),
                mdp_refusals: Some(if rng.random_bool(0.05) { 1 } else { 0 }),
                mdp_failures: Some(if rng.random_bool(0.02) { 1 } else { 0 }),
            });
        }
        d += Duration::days(1);
    }
    ok_response(result)
}

async fn get_robot_data(
    State(st): State<AppState>,
    Query(params): Query<DateRangeParams>,
) -> Json<BaseResponse<Vec<RobotDataResp>>> {
    let mut rng = rand::rngs::SmallRng::from_os_rng();
    let (from, till) = get_date_range(&params);
    let animals = filter_animals(&st, &params);

    let mut result = Vec::new();
    let mut d = from;
    while d <= till {
        for a in &animals {
            let lac = a.calvings.last().map(|c| c.lac_number).unwrap_or(0);
            if lac == 0 {
                continue;
            }
            let days_since_calving = (d - a.calvings.last().unwrap().date).num_days();
            let milk = wood_milk(days_since_calving, lac, &mut rng);
            if milk < 0.5 {
                continue;
            }

            let hour = 6 + rng.random_range(0..10);
            let dt = Utc
                .with_ymd_and_hms(
                    d.year(),
                    d.month(),
                    d.day(),
                    hour as u32,
                    rng.random_range(0..59),
                    0,
                )
                .single()
                .unwrap_or_default();

            result.push(RobotDataResp {
                device_address: Some(rng.random_range(1..=10)),
                life_number: Some(a.life_number.clone()),
                milking_date: fmt_date(dt),
                milk_speed: Some(normal_range(&mut rng, 2.5, 0.8, 1.0, 5.0)),
                milk_speed_max: Some(normal_range(&mut rng, 3.5, 1.0, 1.5, 6.5)),
                lf_milk_time: Some(rng.random_range(120..480)),
                lr_milk_time: Some(rng.random_range(120..480)),
                rf_milk_time: Some(rng.random_range(120..480)),
                rr_milk_time: Some(rng.random_range(120..480)),
                lf_dead_milk_time: Some(rng.random_range(0..30)),
                lr_dead_milk_time: Some(rng.random_range(0..30)),
                rf_dead_milk_time: Some(rng.random_range(0..30)),
                rr_dead_milk_time: Some(rng.random_range(0..30)),
                lf_x_position: Some(rng.random_range(-5..5)),
                lf_y_position: Some(rng.random_range(-5..5)),
                lfz_position: Some(rng.random_range(-3..3)),
                lr_x_position: Some(rng.random_range(-5..5)),
                lr_y_position: Some(rng.random_range(-5..5)),
                lr_z_position: Some(rng.random_range(-3..3)),
                rf_x_position: Some(rng.random_range(-5..5)),
                rf_y_position: Some(rng.random_range(-5..5)),
                rf_z_position: Some(rng.random_range(-3..3)),
                rr_x_position: Some(rng.random_range(-5..5)),
                rr_y_position: Some(rng.random_range(-5..5)),
                rr_z_position: Some(rng.random_range(-3..3)),
            });
        }
        d += Duration::days(1);
    }
    ok_response(result)
}

async fn get_feed_amounts(
    State(st): State<AppState>,
    Query(params): Query<DateRangeParams>,
) -> Json<BaseResponse<Vec<FeedDayAmountResp>>> {
    let mut rng = rand::rngs::SmallRng::from_os_rng();
    let (from, till) = get_date_range(&params);
    let animals = filter_animals(&st, &params);

    let mut result = Vec::new();
    let mut d = from;
    while d <= till {
        for a in &animals {
            let lac = a.calvings.last().map(|c| c.lac_number).unwrap_or(0);
            if lac == 0 {
                continue;
            }
            for feed_num in 1..=2 {
                let total = if feed_num == 1 {
                    normal_range(&mut rng, 25.0, 8.0, 5.0, 45.0)
                } else {
                    normal_range(&mut rng, 8.0, 3.0, 1.0, 15.0)
                };
                result.push(FeedDayAmountResp {
                    life_number: a.life_number.clone(),
                    feed_date: fmt_date_only(d),
                    feed_number: Some(feed_num),
                    total: Some((total * 10.0).round() / 10.0),
                    rest_feed: Some(rng.random_range(0..=5)),
                });
            }
        }
        d += Duration::days(1);
    }
    ok_response(result)
}

async fn get_feed_visits(
    State(st): State<AppState>,
    Query(params): Query<DateRangeParams>,
) -> Json<BaseResponse<Vec<FeedVisitResp>>> {
    let mut rng = rand::rngs::SmallRng::from_os_rng();
    let (from, till) = get_date_range(&params);
    let animals = filter_animals(&st, &params);

    let mut result = Vec::new();
    let mut d = from;
    while d <= till {
        for a in &animals {
            let num_visits = rng.random_range(3..=8);
            for _ in 0..num_visits {
                let hour = rng.random_range(5..=21);
                let dt = Utc
                    .with_ymd_and_hms(
                        d.year(),
                        d.month(),
                        d.day(),
                        hour as u32,
                        rng.random_range(0..59),
                        0,
                    )
                    .single()
                    .unwrap_or_default();

                result.push(FeedVisitResp {
                    device_address: Some(rng.random_range(1..=10)),
                    life_number: a.life_number.clone(),
                    feed_date: fmt_date(dt),
                    feed_name: Some(
                        if rng.random_bool(0.7) {
                            "Концентрат"
                        } else {
                            "Премікс"
                        }
                        .to_string(),
                    ),
                    number_of_feed_type: Some(rng.random_range(1..=3)),
                    credit: Some(rng.random_range(500..3000)),
                    intake: Some(rng.random_range(300..2500)),
                });
            }
        }
        d += Duration::days(1);
    }
    ok_response(result)
}

async fn get_activities(
    State(st): State<AppState>,
    Query(params): Query<DateRangeParams>,
) -> Json<BaseResponse<Vec<ActivityResp>>> {
    let mut rng = rand::rngs::SmallRng::from_os_rng();
    let (from, till) = get_date_range(&params);
    let animals = filter_animals(&st, &params);

    let mut result = Vec::new();
    let mut d = from;
    while d <= till {
        for a in &animals {
            let hour = rng.random_range(6..=20);
            let dt = Utc
                .with_ymd_and_hms(d.year(), d.month(), d.day(), hour as u32, 0, 0)
                .single()
                .unwrap_or_default();
            result.push(ActivityResp {
                life_number: a.life_number.clone(),
                activity_date_time: fmt_date(dt),
                activity_counter: Some(normal_range(&mut rng, 50.0, 20.0, 10.0, 100.0) as i32),
                heat_attention: Some(rng.random_bool(0.03)),
            });
        }
        d += Duration::days(1);
    }
    ok_response(result)
}

async fn get_ruminations(
    State(st): State<AppState>,
    Query(params): Query<DateRangeParams>,
) -> Json<BaseResponse<Vec<RuminationResp>>> {
    let mut rng = rand::rngs::SmallRng::from_os_rng();
    let (from, till) = get_date_range(&params);
    let animals = filter_animals(&st, &params);

    let mut result = Vec::new();
    let mut d = from;
    while d <= till {
        for a in &animals {
            let hour = 12;
            let dt = Utc
                .with_ymd_and_hms(d.year(), d.month(), d.day(), hour, 0, 0)
                .single()
                .unwrap_or_default();
            result.push(RuminationResp {
                life_number: a.life_number.clone(),
                date_time: fmt_date(dt),
                eating_seconds: Some(
                    normal_range(&mut rng, 18000.0, 5000.0, 8000.0, 35000.0) as i32
                ),
                rumination_minutes: Some(normal_range(&mut rng, 450.0, 120.0, 200.0, 700.0) as i32),
            });
        }
        d += Duration::days(1);
    }
    ok_response(result)
}

async fn get_grazing(
    State(st): State<AppState>,
    Query(params): Query<DateRangeParams>,
) -> Json<BaseResponse<Vec<GrazingResp>>> {
    let mut rng = rand::rngs::SmallRng::from_os_rng();
    let (from, till) = get_date_range(&params);

    let total_cows = st.animals.len() as i32;
    let mut result = Vec::new();
    let mut cum_days = 0i32;
    let mut cum_time = 0i32;
    let mut d = from;
    while d <= till {
        let is_grazing_day = d.month() >= 5 && d.month() <= 9 && rng.random_bool(0.7);
        let pct = if is_grazing_day {
            normal_range(&mut rng, 80.0, 15.0, 40.0, 98.0)
        } else {
            normal_range(&mut rng, 10.0, 15.0, 0.0, 30.0)
        };
        if is_grazing_day {
            cum_days += 1;
        }
        let pasture_time = if is_grazing_day {
            rng.random_range(360..720)
        } else {
            0
        };
        cum_time += pasture_time;

        result.push(GrazingResp {
            date: fmt_date_only(d),
            total_milking_cows: total_cows,
            cows14_dil: (total_cows as f64 * 0.15) as i32,
            percentage_in_pasture: (pct * 10.0).round() / 10.0,
            grazing_day_yes_no: pct > 90.0,
            grazing_time: pasture_time,
            sd_time_pasture: rng.random_range(30..120),
            cum_pasture_days: cum_days,
            cum_total_pasturetime: cum_time,
            tank_number: rng.random_range(1..=3) as i64,
        });
        d += Duration::days(1);
    }
    ok_response(result)
}

async fn get_calvings(
    State(st): State<AppState>,
    Query(params): Query<DateRangeParams>,
) -> Json<BaseResponse<Vec<CalvingResp>>> {
    let (from, till) = get_date_range(&params);
    let animals = filter_animals(&st, &params);

    let mut result = Vec::new();
    for a in &animals {
        for c in &a.calvings {
            if c.date >= from && c.date <= till {
                result.push(CalvingResp {
                    life_number: a.life_number.clone(),
                    calving_date: fmt_date_only(c.date),
                    remarks: Some("Normal".to_string()),
                    lac_number: Some(c.lac_number),
                    calves: Some(vec![CalfResp {
                        gender: "Female".to_string(),
                        birth_remark: Some("Normal".to_string()),
                        keep: Some(true),
                        weight: Some(normal_range(
                            &mut rand::rngs::SmallRng::from_os_rng(),
                            38.0,
                            5.0,
                            25.0,
                            55.0,
                        )),
                        born_dead: Some(false),
                        calf_name: None,
                        hair_color_code: Some("RH".to_string()),
                    }]),
                });
            }
        }
    }
    ok_response(result)
}

async fn get_inseminations(
    State(st): State<AppState>,
    Query(params): Query<DateRangeParams>,
) -> Json<BaseResponse<Vec<InseminationResp>>> {
    let mut rng = rand::rngs::SmallRng::from_os_rng();
    let (from, till) = get_date_range(&params);
    let animals = filter_animals(&st, &params);

    let mut result = Vec::new();
    for a in &animals {
        for c in &a.calvings {
            let ins_date = c.date - Duration::days(280);
            if ins_date >= from && ins_date <= till {
                result.push(InseminationResp {
                    life_number: a.life_number.clone(),
                    insemination_date: fmt_date_only(ins_date),
                    insemination_type: "AI".to_string(),
                    sire_code: Some(pick(&mut rng, SIRE_CODES).to_string()),
                    charge_number: Some(format!("CH{:06}", rng.random_range(1u32..999999))),
                    insemination_number: Some(rng.random_range(1..=5)),
                });
            }
        }
    }
    ok_response(result)
}

async fn get_pregnancies(
    State(st): State<AppState>,
    Query(params): Query<DateRangeParams>,
) -> Json<BaseResponse<Vec<PregnancyResp>>> {
    let (from, till) = get_date_range(&params);
    let animals = filter_animals(&st, &params);

    let mut result = Vec::new();
    for a in &animals {
        for c in &a.calvings {
            let preg_date = c.date - Duration::days(250);
            let ins_date = c.date - Duration::days(280);
            if preg_date >= from && preg_date <= till {
                result.push(PregnancyResp {
                    life_number: a.life_number.clone(),
                    pregnancy_date: fmt_date_only(preg_date),
                    pregnancy_type: "Pregnant".to_string(),
                    insemination_date: Some(fmt_date_only(ins_date)),
                });
            }
        }
    }
    ok_response(result)
}

async fn get_heats(
    State(st): State<AppState>,
    Query(params): Query<DateRangeParams>,
) -> Json<BaseResponse<Vec<HeatResp>>> {
    let (from, till) = get_date_range(&params);
    let animals = filter_animals(&st, &params);

    let mut result = Vec::new();
    for a in &animals {
        for c in &a.calvings {
            let heat_date = c.date - Duration::days(295);
            if heat_date >= from && heat_date <= till {
                result.push(HeatResp {
                    life_number: a.life_number.clone(),
                    heat_date: fmt_date_only(heat_date),
                });
            }
        }
    }
    ok_response(result)
}

async fn get_dry_offs(
    State(st): State<AppState>,
    Query(params): Query<DateRangeParams>,
) -> Json<BaseResponse<Vec<DryOffResp>>> {
    let (from, till) = get_date_range(&params);
    let animals = filter_animals(&st, &params);

    let mut result = Vec::new();
    for a in &animals {
        for c in &a.calvings {
            let dry_date = c.date - Duration::days(60);
            if dry_date >= from && dry_date <= till {
                result.push(DryOffResp {
                    life_number: a.life_number.clone(),
                    dry_off_date: fmt_date_only(dry_date),
                });
            }
        }
    }
    ok_response(result)
}

async fn get_sires(State(_st): State<AppState>) -> Json<BaseResponse<Vec<SireResp>>> {
    let data: Vec<SireResp> = SIRE_CODES
        .iter()
        .enumerate()
        .map(|(i, code)| SireResp {
            sire_code: code.to_string(),
            sire_name: Some(format!("Бик #{}", i + 1)),
            life_number: format!("SIRE{:012}", i),
            sire_active: Some(true),
        })
        .collect();
    ok_response(data)
}

async fn get_transfers(
    State(st): State<AppState>,
    Query(params): Query<DateRangeParams>,
) -> Json<BaseResponse<Vec<TransferResp>>> {
    let (from, till) = get_date_range(&params);
    let animals = filter_animals(&st, &params);

    let mut result = Vec::new();
    for a in &animals {
        let transfer_date = a.birth_date;
        if transfer_date >= from && transfer_date <= till {
            result.push(TransferResp {
                life_number: a.life_number.clone(),
                transfer_date: fmt_date_only(transfer_date),
                transfer_type: "Birth".to_string(),
                reason_id: None,
            });
        }
    }
    ok_response(result)
}

#[derive(Deserialize)]
struct BloodlineParams {
    life_number: Option<String>,
}

async fn get_bloodlines(
    State(st): State<AppState>,
    Query(params): Query<BloodlineParams>,
) -> Json<BaseResponse<Vec<BloodLineResp>>> {
    let mut rng = rand::rngs::SmallRng::from_os_rng();
    let animals: Vec<&AnimalData> = if let Some(ref ln) = params.life_number {
        st.animals.iter().filter(|a| a.life_number == *ln).collect()
    } else {
        st.animals.iter().take(10).collect()
    };

    let codes: &[&str] = &["HOL", "RBT", "JER"];
    let mut result = Vec::new();
    for a in &animals {
        let code = pick(&mut rng, codes).to_string();
        let pct = normal_range(&mut rng, 85.0, 10.0, 50.0, 100.0);
        result.push(BloodLineResp {
            life_number: a.life_number.clone(),
            blood_type_code: code,
            percentage: (pct * 10.0).round() / 10.0,
        });
    }
    ok_response(result)
}

async fn get_contacts(State(_st): State<AppState>) -> Json<BaseResponse<Vec<ContactResp>>> {
    let data = vec![
        ContactResp {
            contact_id: 1,
            name: Some("Іванов Іван".to_string()),
            contact_type_id: 1,
            contact_type_name: Some("Farmer".to_string()),
            farm_number: Some("F001".to_string()),
            phone_cell: Some("+380 67 123-45-67".to_string()),
            email: Some("ivanov@farm.ua".to_string()),
            active: Some(true),
        },
        ContactResp {
            contact_id: 2,
            name: Some("Петренко Ветеринар".to_string()),
            contact_type_id: 6,
            contact_type_name: Some("Veterinarian".to_string()),
            farm_number: None,
            phone_cell: Some("+380 50 987-65-43".to_string()),
            email: Some("vet@clinic.ua".to_string()),
            active: Some(true),
        },
    ];
    ok_response(data)
}

async fn get_locations(State(_st): State<AppState>) -> Json<BaseResponse<Vec<LocationResp>>> {
    let data = vec![
        LocationResp {
            location_id: 1,
            location_name: Some("Корівник 1".to_string()),
            location_description: Some("Основний корівник".to_string()),
            barn_id: Some(1),
            feed_group_id: Some(1),
        },
        LocationResp {
            location_id: 2,
            location_name: Some("Корівник 2".to_string()),
            location_description: Some("Другий корівник".to_string()),
            barn_id: Some(2),
            feed_group_id: Some(2),
        },
    ];
    ok_response(data)
}

async fn get_feed_types(State(_st): State<AppState>) -> Json<BaseResponse<Vec<FeedTypeResp>>> {
    let data = vec![
        FeedTypeResp {
            feed_id: 1,
            number_of_feed_type: 1,
            feed_type: "Concentrate".to_string(),
            feed_name: "Концентрат Стандарт".to_string(),
            feed_dry_matter_percentage: 88.0,
            feed_price: 12.5,
        },
        FeedTypeResp {
            feed_id: 2,
            number_of_feed_type: 2,
            feed_type: "Mineral".to_string(),
            feed_name: "Мінеральний премікс".to_string(),
            feed_dry_matter_percentage: 95.0,
            feed_price: 25.0,
        },
    ];
    ok_response(data)
}

async fn get_feed_groups(State(_st): State<AppState>) -> Json<BaseResponse<Vec<FeedGroupResp>>> {
    let data = vec![
        FeedGroupResp {
            feed_group_id: Some("1".to_string()),
            feed_group_name: Some("Висока продуктивність".to_string()),
            max_milk_yield: 45.0,
            min_milk_yield: 30.0,
            average_milk_yield: 35.0,
            number_of_cows: 80,
        },
        FeedGroupResp {
            feed_group_id: Some("2".to_string()),
            feed_group_name: Some("Середня продуктивність".to_string()),
            max_milk_yield: 30.0,
            min_milk_yield: 15.0,
            average_milk_yield: 22.0,
            number_of_cows: 120,
        },
    ];
    ok_response(data)
}

use clap::Parser;

#[derive(Parser)]
#[command(
    name = "mock-lely",
    about = "Mock Lely Business Integration API сервер"
)]
struct Args {
    #[arg(long, default_value_t = 1988)]
    port: u16,

    #[arg(long, default_value_t = 50)]
    cows: usize,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    tracing_subscriber::fmt()
        .with_env_filter("mock_lely=info")
        .init();

    let animals = generate_animals(args.cows);
    let today = Utc::now().date_naive();

    let state = Arc::new(MockState {
        animals,
        start_date: today - Duration::days(30),
        end_date: today,
    });

    let app = make_router(state);

    let addr = format!("0.0.0.0:{}", args.port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("🤖 Mock Lely API сервер запущено на http://{}", addr);
    tracing::info!("   FarmKey: будь-яке значення");
    tracing::info!("   Username/Password: будь-які значення");
    tracing::info!("   Коров: {}", args.cows);

    axum::serve(listener, app).await.unwrap();
}
