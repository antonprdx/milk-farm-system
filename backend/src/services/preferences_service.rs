use sqlx::PgPool;

use crate::errors::AppError;
use crate::models::preferences::{UpdatePreferences, UserPreferences};
use serde_json::Value;

pub async fn get(pool: &PgPool, user_id: i32) -> Result<UserPreferences, AppError> {
    let row = sqlx::query_as::<_, (String, i32, bool, String, Option<Value>)>(
        "SELECT theme, page_size, compact_view, language, dashboard_widgets FROM user_preferences WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?;

    Ok(row
        .map(|(theme, page_size, compact_view, language, dashboard_widgets)| UserPreferences {
            theme,
            page_size,
            compact_view,
            language,
            dashboard_widgets: dashboard_widgets.unwrap_or_else(|| {
                serde_json::from_str(
                    "[\"kpi\",\"milk_trend\",\"alerts\",\"reproduction\",\"feed\",\"latest_milk\",\"system_status\",\"vet_followups\",\"active_withdrawals\",\"overdue_tasks\"]",
                )
                .unwrap_or(Value::Null)
            }),
        })
        .unwrap_or_default())
}

pub async fn update(
    pool: &PgPool,
    user_id: i32,
    req: &UpdatePreferences,
) -> Result<UserPreferences, AppError> {
    let current = get(pool, user_id).await?;
    let theme = req.theme.as_deref().unwrap_or(&current.theme);
    let page_size = req.page_size.unwrap_or(current.page_size);
    let compact_view = req.compact_view.unwrap_or(current.compact_view);
    let language = req.language.as_deref().unwrap_or(&current.language);
    let dashboard_widgets = req
        .dashboard_widgets
        .as_ref()
        .unwrap_or(&current.dashboard_widgets)
        .clone();

    if !(5..=200).contains(&page_size) {
        return Err(AppError::BadRequest("page_size must be 5-200".into()));
    }
    if !["light", "dark", "auto"].contains(&theme) {
        return Err(AppError::BadRequest(
            "theme must be light, dark, or auto".into(),
        ));
    }
    if !["ru", "en"].contains(&language) {
        return Err(AppError::BadRequest("language must be ru or en".into()));
    }

    let row = sqlx::query_as::<_, (String, i32, bool, String, Value)>(
        "INSERT INTO user_preferences (user_id, theme, page_size, compact_view, language, dashboard_widgets)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (user_id) DO UPDATE SET theme = $2, page_size = $3, compact_view = $4, language = $5, dashboard_widgets = $6, updated_at = NOW()
         RETURNING theme, page_size, compact_view, language, dashboard_widgets",
    )
    .bind(user_id)
    .bind(theme)
    .bind(page_size)
    .bind(compact_view)
    .bind(language)
    .bind(&dashboard_widgets)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)?;

    Ok(UserPreferences {
        theme: row.0,
        page_size: row.1,
        compact_view: row.2,
        language: row.3,
        dashboard_widgets: row.4,
    })
}
