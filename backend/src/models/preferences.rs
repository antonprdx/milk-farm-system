use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserPreferences {
    pub theme: String,
    pub page_size: i32,
    pub compact_view: bool,
    pub language: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdatePreferences {
    pub theme: Option<String>,
    pub page_size: Option<i32>,
    pub compact_view: Option<bool>,
    pub language: Option<String>,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            theme: "light".into(),
            page_size: 20,
            compact_view: false,
            language: "ru".into(),
        }
    }
}
