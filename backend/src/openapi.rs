use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::Modify;
use utoipa::OpenApi;

use crate::handlers::{ChangePasswordRequest, ReportFilter, TrendQuery, UpdateRoleRequest};
use crate::middleware::auth::{Claims, LoginRequest, RegisterRequest};
use crate::models::analytics::{
    Alert, AlertsResponse, CullingRiskEntry, DailyMilkPoint, DryOffRecommendation, ExpectedCalving,
    ExpectedHeat, FeedForecastResponse, ForecastPoint, KpiResponse, LactationAvg,
    MilkTrendResponse, ReproductionForecastResponse,
};
use crate::models::animal::{Animal, AnimalFilter, CreateAnimal, UpdateAnimal};
use crate::models::animal_stats::{
    AnimalStats, LatestMetrics, MilkDataPoint, ReproductionSummary, SccDataPoint,
};
use crate::models::bulk_tank::{
    BulkTankFilter, BulkTankTest, CreateBulkTankTest, UpdateBulkTankTest,
};
use crate::models::contact::{Contact, ContactFilter, CreateContact, UpdateContact};
use crate::models::feed::{
    CreateFeedDayAmount, FeedDayAmount, FeedFilter, FeedGroup, FeedType, FeedVisit,
};
use crate::models::fitness::{Activity, FitnessFilter, Rumination};
use crate::models::grazing::{GrazingData, GrazingFilter};
use crate::models::location::Location;
use crate::models::milk::{
    CreateMilkDayProduction, MilkDayProduction, MilkFilter, MilkQuality, MilkVisit,
    UpdateMilkDayProduction,
};
use crate::models::pagination::Pagination;
use crate::models::preferences::{UpdatePreferences, UserPreferences};
use crate::models::reproduction::{
    Calf, Calving, CreateCalf, CreateCalving, CreateDryOff, CreateHeat, CreateInsemination,
    CreatePregnancy, DryOff, Heat, Insemination, Pregnancy, ReproductionFilter, UpdateCalving,
    UpdateDryOff, UpdateHeat, UpdateInsemination, UpdatePregnancy,
};
use crate::models::system_settings::{
    AlertThresholds, JwtTtlSettings, SystemInfo, SystemSetting, UpdateAlertThresholds,
    UpdateJwtTtl, UpdateSystemSetting,
};
use crate::models::timeline::{TimelineEvent, TimelineResponse};
use crate::models::user::UserPublic;
use crate::models::{BirthRemarkType, GenderType};

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "cookie_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::auth::health,
        crate::handlers::auth::liveness,
        crate::handlers::auth::readiness,
        crate::handlers::auth::login,
        crate::handlers::auth::logout,
        crate::handlers::auth::register,
        crate::handlers::auth::refresh,
        crate::handlers::auth::stats,
        crate::handlers::animals::list,
        crate::handlers::animals::get_by_id,
        crate::handlers::animals::create,
        crate::handlers::animals::update,
        crate::handlers::animals::remove,
        crate::handlers::animals::timeline,
        crate::handlers::animals::stats,
        crate::handlers::milk::list_productions,
        crate::handlers::milk::create_production,
        crate::handlers::milk::get_production,
        crate::handlers::milk::update_production,
        crate::handlers::milk::delete_production,
        crate::handlers::milk::list_visits,
        crate::handlers::milk::list_quality,
        crate::handlers::reproduction::list_calvings,
        crate::handlers::reproduction::get_calving,
        crate::handlers::reproduction::create_calving,
        crate::handlers::reproduction::update_calving,
        crate::handlers::reproduction::delete_calving,
        crate::handlers::reproduction::list_inseminations,
        crate::handlers::reproduction::get_insemination,
        crate::handlers::reproduction::create_insemination,
        crate::handlers::reproduction::update_insemination,
        crate::handlers::reproduction::delete_insemination,
        crate::handlers::reproduction::list_pregnancies,
        crate::handlers::reproduction::get_pregnancy,
        crate::handlers::reproduction::create_pregnancy,
        crate::handlers::reproduction::update_pregnancy,
        crate::handlers::reproduction::delete_pregnancy,
        crate::handlers::reproduction::list_heats,
        crate::handlers::reproduction::get_heat,
        crate::handlers::reproduction::create_heat,
        crate::handlers::reproduction::update_heat,
        crate::handlers::reproduction::delete_heat,
        crate::handlers::reproduction::list_dryoffs,
        crate::handlers::reproduction::get_dryoff,
        crate::handlers::reproduction::create_dryoff,
        crate::handlers::reproduction::update_dryoff,
        crate::handlers::reproduction::delete_dryoff,
        crate::handlers::reproduction::current_status,
        crate::handlers::feed::list_day_amounts,
        crate::handlers::feed::list_visits,
        crate::handlers::feed::list_types,
        crate::handlers::feed::list_groups,
        crate::handlers::fitness::list_activities,
        crate::handlers::fitness::list_ruminations,
        crate::handlers::grazing::list_grazing,
        crate::handlers::bulk_tank::list,
        crate::handlers::bulk_tank::get_by_id,
        crate::handlers::bulk_tank::create,
        crate::handlers::bulk_tank::update,
        crate::handlers::bulk_tank::remove,
        crate::handlers::contacts::list,
        crate::handlers::contacts::create,
        crate::handlers::contacts::update,
        crate::handlers::contacts::remove,
        crate::handlers::reports::milk_summary,
        crate::handlers::reports::reproduction_summary,
        crate::handlers::reports::feed_summary,
        crate::handlers::reports::export_milk,
        crate::handlers::reports::export_reproduction,
        crate::handlers::reports::export_feed,
        crate::handlers::reports::export_milk_pdf,
        crate::handlers::reports::export_reproduction_pdf,
        crate::handlers::reports::export_feed_pdf,
        crate::handlers::settings::list_users,
        crate::handlers::settings::create_user,
        crate::handlers::settings::change_password,
        crate::handlers::settings::delete_user,
        crate::handlers::settings::update_role,
        crate::handlers::settings::get_preferences,
        crate::handlers::settings::update_preferences,
        crate::handlers::settings::system_info,
        crate::handlers::settings::get_jwt_ttl,
        crate::handlers::settings::update_jwt_ttl,
        crate::handlers::settings::get_alert_thresholds,
        crate::handlers::settings::update_alert_thresholds,
        crate::handlers::settings::backup_database,
        crate::handlers::analytics::kpi,
        crate::handlers::analytics::alerts,
        crate::handlers::analytics::milk_trend,
        crate::handlers::analytics::reproduction_forecast,
        crate::handlers::analytics::feed_forecast,
        crate::handlers::locations::list
    ),
    components(schemas(
        GenderType,
        BirthRemarkType,
        Animal,
        CreateAnimal,
        UpdateAnimal,
        AnimalFilter,
        AnimalStats,
        MilkDataPoint,
        SccDataPoint,
        LatestMetrics,
        ReproductionSummary,
        MilkDayProduction,
        MilkVisit,
        MilkQuality,
        CreateMilkDayProduction,
        UpdateMilkDayProduction,
        MilkFilter,
        Calving,
        Calf,
        Insemination,
        Pregnancy,
        Heat,
        DryOff,
        CreateCalving,
        CreateCalf,
        CreateInsemination,
        CreatePregnancy,
        CreateHeat,
        CreateDryOff,
        UpdateCalving,
        UpdateInsemination,
        UpdatePregnancy,
        UpdateHeat,
        UpdateDryOff,
        ReproductionFilter,
        FeedDayAmount,
        FeedVisit,
        FeedType,
        FeedGroup,
        CreateFeedDayAmount,
        FeedFilter,
        Activity,
        Rumination,
        FitnessFilter,
        GrazingData,
        GrazingFilter,
        Contact,
        CreateContact,
        UpdateContact,
        ContactFilter,
        BulkTankTest,
        CreateBulkTankTest,
        UpdateBulkTankTest,
        BulkTankFilter,
        Location,
        TimelineEvent,
        TimelineResponse,
        KpiResponse,
        LactationAvg,
        CullingRiskEntry,
        Alert,
        AlertsResponse,
        DailyMilkPoint,
        ForecastPoint,
        MilkTrendResponse,
        ExpectedCalving,
        ExpectedHeat,
        DryOffRecommendation,
        ReproductionForecastResponse,
        FeedForecastResponse,
        Pagination,
        UserPublic,
        Claims,
        LoginRequest,
        RegisterRequest,
        ChangePasswordRequest,
        UpdateRoleRequest,
        ReportFilter,
        TrendQuery,
        UserPreferences,
        UpdatePreferences,
        SystemSetting,
        UpdateSystemSetting,
        SystemInfo,
        AlertThresholds,
        UpdateAlertThresholds,
        JwtTtlSettings,
        UpdateJwtTtl
    )),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;
