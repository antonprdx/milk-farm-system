use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

const DEFAULT_DASHBOARD_WIDGETS: &str = "[\"kpi\",\"milk_trend\",\"alerts\",\"reproduction\",\"feed\",\"latest_milk\",\"system_status\",\"vet_followups\",\"active_withdrawals\",\"overdue_tasks\"]";

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserPreferences {
    pub theme: String,
    pub page_size: i32,
    pub compact_view: bool,
    pub language: String,
    #[serde(default = "default_dashboard_widgets")]
    pub dashboard_widgets: Value,
}

fn default_dashboard_widgets() -> Value {
    serde_json::from_str(DEFAULT_DASHBOARD_WIDGETS).unwrap_or(Value::Null)
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdatePreferences {
    pub theme: Option<String>,
    pub page_size: Option<i32>,
    pub compact_view: Option<bool>,
    pub language: Option<String>,
    pub dashboard_widgets: Option<Value>,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            theme: "light".into(),
            page_size: 20,
            compact_view: false,
            language: "ru".into(),
            dashboard_widgets: default_dashboard_widgets(),
        }
    }
}
