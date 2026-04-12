use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct LelyMeta {
    #[allow(dead_code)]
    pub code: i32,
    #[allow(dead_code)]
    pub message: Option<String>,
    #[allow(dead_code)]
    pub errors: Option<serde_json::Value>,
    #[allow(dead_code)]
    pub detail: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct LelyResponse<T> {
    pub data: Option<T>,
    pub meta: LelyMeta,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct LelyListResponse<T> {
    pub data: Option<Vec<T>>,
    pub meta: LelyMeta,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct AuthenticateRequest {
    pub user_name: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AnimalResponse {
    pub ani_id: Option<i32>,
    pub life_number: Option<String>,
    pub name: Option<String>,
    pub user_number: Option<i64>,
    pub gender: Option<String>,
    pub birth_date: Option<String>,
    pub hair_color_code: Option<String>,
    pub father_life_number: Option<String>,
    pub mother_life_number: Option<String>,
    pub description: Option<String>,
    pub ucn_number: Option<String>,
    pub use_as_sire: Option<bool>,
    pub location: Option<String>,
    pub group_number: Option<i32>,
    pub keep: Option<bool>,
    pub gestation: Option<i32>,
    pub responder_number: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DayProduction {
    pub life_number: Option<String>,
    pub date: Option<String>,
    pub milk_day_production: Option<f64>,
    pub milk_day_production_average: Option<f64>,
    pub average_weight: Option<f64>,
    pub isk: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MilkVisit {
    pub life_number: Option<String>,
    pub milking_date: Option<String>,
    pub milking_start_date: Option<String>,
    pub milk_yield: Option<f64>,
    pub bottle_number: Option<i32>,
    pub success_milking: Option<bool>,
    pub weight: Option<i32>,
    pub device_address: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MilkVisitQuality {
    pub life_number: Option<String>,
    pub milking_date: Option<String>,
    pub device_address: Option<i32>,
    pub milking_start_date: Option<String>,
    pub success_milking: Option<bool>,
    pub milk_yield: Option<f64>,
    pub bottle_number: Option<i32>,
    pub milk_temperature: Option<f64>,
    pub weight: Option<i32>,
    pub lf_colour_code: Option<String>,
    pub lr_colour_code: Option<String>,
    pub rf_colour_code: Option<String>,
    pub rr_colour_code: Option<String>,
    pub lf_conductivity: Option<i32>,
    pub lr_conductivity: Option<i32>,
    pub rf_conductivity: Option<i32>,
    pub rr_conductivity: Option<i32>,
    pub milk_destination: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DayProductionQuality {
    pub life_number: Option<String>,
    pub date: Option<String>,
    pub milk_day_production: Option<f64>,
    pub milk_day_production_average: Option<f64>,
    pub average_weight: Option<f64>,
    pub isk: Option<f64>,
    pub fat_percentage: Option<f64>,
    pub protein_percentage: Option<f64>,
    pub lactose_percentage: Option<f64>,
    pub scc: Option<i32>,
    pub mdp_milkings: Option<i32>,
    pub mdp_refusals: Option<i32>,
    pub mdp_failures: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FeedDayAmount {
    pub life_number: Option<String>,
    pub feed_date: Option<String>,
    pub feed_number: Option<i32>,
    pub total: Option<f64>,
    pub rest_feed: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FeedVisit {
    pub device_address: Option<i32>,
    pub life_number: Option<String>,
    pub feed_date: Option<String>,
    pub feed_name: Option<String>,
    pub number_of_feed_type: Option<i32>,
    pub credit: Option<i32>,
    pub intake: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Activity {
    pub life_number: Option<String>,
    pub activity_date_time: Option<String>,
    pub activity_counter: Option<i32>,
    pub heat_attention: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Rumination {
    pub life_number: Option<String>,
    pub date_time: Option<String>,
    pub eating_seconds: Option<i32>,
    pub rumination_minutes: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Grazing {
    pub date: Option<String>,
    pub total_milking_cows: Option<i32>,
    pub cows14_dil: Option<i32>,
    pub percentage_in_pasture: Option<f64>,
    pub grazing_day_yes_no: Option<bool>,
    pub grazing_time: Option<i32>,
    pub sd_time_pasture: Option<i32>,
    pub cum_pasture_days: Option<i32>,
    pub cum_total_pasturetime: Option<i32>,
    pub tank_number: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RobotData {
    pub device_address: Option<i32>,
    pub life_number: Option<String>,
    pub milking_date: Option<String>,
    pub milk_speed: Option<f64>,
    pub milk_speed_max: Option<f64>,
    pub lf_milk_time: Option<i32>,
    pub lr_milk_time: Option<i32>,
    pub rf_milk_time: Option<i32>,
    pub rr_milk_time: Option<i32>,
    pub lf_dead_milk_time: Option<i32>,
    pub lr_dead_milk_time: Option<i32>,
    pub rf_dead_milk_time: Option<i32>,
    pub rr_dead_milk_time: Option<i32>,
    pub lf_x_position: Option<i32>,
    pub lf_y_position: Option<i32>,
    pub lfz_position: Option<i32>,
    pub lr_x_position: Option<i32>,
    pub lr_y_position: Option<i32>,
    pub lr_z_position: Option<i32>,
    pub rf_x_position: Option<i32>,
    pub rf_y_position: Option<i32>,
    pub rf_z_position: Option<i32>,
    pub rr_x_position: Option<i32>,
    pub rr_y_position: Option<i32>,
    pub rr_z_position: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Calving {
    pub life_number: Option<String>,
    pub calving_date: Option<String>,
    pub remarks: Option<String>,
    pub lac_number: Option<i32>,
    pub calves: Option<Vec<Calf>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Calf {
    pub ani_id: Option<i32>,
    pub life_number: Option<String>,
    pub birth_remark: Option<String>,
    pub keep: Option<bool>,
    pub weight: Option<f64>,
    pub born_dead: Option<bool>,
    pub gender: Option<String>,
    pub animal_number: Option<i64>,
    pub calf_name: Option<String>,
    pub hair_color_code: Option<String>,
    pub born_dead_reason_id: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Insemination {
    pub life_number: Option<String>,
    pub insemination_date: Option<String>,
    pub insemination_type: Option<String>,
    pub sire_code: Option<String>,
    pub sir_life_number: Option<String>,
    pub contact_name: Option<String>,
    pub remarks: Option<String>,
    pub contact_id: Option<i32>,
    pub charge_number: Option<String>,
    pub insemination_number: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Pregnancy {
    pub life_number: Option<String>,
    pub pregnancy_date: Option<String>,
    pub pregnancy_type: Option<String>,
    pub insemination_date: Option<String>,
    pub person_id: Option<i32>,
    pub person_name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Heat {
    pub life_number: Option<String>,
    pub heat_date: Option<String>,
    pub heat_remark: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DryOff {
    pub life_number: Option<String>,
    pub dry_off_date: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Sire {
    pub sire_code: Option<String>,
    pub sire_name: Option<String>,
    pub life_number: Option<String>,
    pub sire_active: Option<bool>,
    pub sire_description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Transfer {
    pub life_number: Option<String>,
    pub transfer_date: Option<String>,
    pub transfer_remarks: Option<String>,
    pub transfer_type: Option<String>,
    pub herd_ucn_number: Option<String>,
    pub ucn_origin: Option<String>,
    pub ucn_destination: Option<String>,
    pub reason_id: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BloodLine {
    pub life_number: Option<String>,
    pub blood_type_code: Option<String>,
    pub percentage: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FeedType {
    pub feed_id: Option<i32>,
    pub number_of_feed_type: Option<i32>,
    pub feed_type: Option<String>,
    pub feed_name: Option<String>,
    pub feed_description: Option<String>,
    pub feed_dry_matter_percentage: Option<f64>,
    pub stock_attention_level: Option<i32>,
    pub feed_price: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FeedGroup {
    pub feed_group_id: Option<String>,
    pub feed_group_name: Option<String>,
    pub max_milk_yield: Option<f64>,
    pub min_milk_yield: Option<f64>,
    pub average_milk_yield: Option<f64>,
    pub average_milk_fat: Option<f64>,
    pub average_milk_protein: Option<f64>,
    pub average_weight: Option<f64>,
    pub max_number_of_robot_feed_types: Option<i32>,
    pub max_feed_intake_robot: Option<f64>,
    pub min_feed_intake_robot: Option<f64>,
    pub number_of_cows: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Contact {
    pub contact_id: Option<i32>,
    pub name: Option<String>,
    pub contact_type_id: Option<i32>,
    pub contact_type_name: Option<String>,
    pub farm_number: Option<String>,
    pub phone_cell: Option<String>,
    pub phone_home: Option<String>,
    pub phone_work: Option<String>,
    pub phone_fax: Option<String>,
    pub email: Option<String>,
    pub company_name: Option<String>,
    pub description: Option<String>,
    pub street_name: Option<String>,
    pub street_name_ext: Option<String>,
    pub postal_code: Option<String>,
    pub country_id: Option<i32>,
    pub active: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Location {
    pub location_id: Option<i32>,
    pub location_name: Option<String>,
    pub location_description: Option<String>,
    pub barn_id: Option<i32>,
    pub feed_group_id: Option<i32>,
}

pub fn parse_date(s: &Option<String>) -> Option<chrono::NaiveDate> {
    s.as_ref().and_then(|d| {
        chrono::DateTime::parse_from_rfc3339(d)
            .map(|dt| dt.date_naive())
            .or_else(|_| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d"))
            .ok()
    })
}

pub fn parse_datetime(s: &Option<String>) -> Option<chrono::DateTime<chrono::Utc>> {
    s.as_ref().and_then(|d| {
        chrono::DateTime::parse_from_rfc3339(d)
            .ok()
            .map(|dt| dt.to_utc())
    })
}

pub use parse_date as parse_lely_date;
pub use parse_datetime as parse_lely_datetime;
