use sqlx::PgPool;

use crate::errors::AppError;
use crate::models::preferences::{UpdatePreferences, UserPreferences};

pub async fn get(pool: &PgPool, user_id: i32) -> Result<UserPreferences, AppError> {
    let row = sqlx::query_as::<_, (String, i32, bool, String)>(
        "SELECT theme, page_size, compact_view, language FROM user_preferences WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?;

    Ok(row
        .map(
            |(theme, page_size, compact_view, language)| UserPreferences {
                theme,
                page_size,
                compact_view,
                language,
            },
        )
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

    let row = sqlx::query_as::<_, (String, i32, bool, String)>(
        "INSERT INTO user_preferences (user_id, theme, page_size, compact_view, language)
         VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (user_id) DO UPDATE SET theme = $2, page_size = $3, compact_view = $4, language = $5, updated_at = NOW()
         RETURNING theme, page_size, compact_view, language",
    )
    .bind(user_id)
    .bind(theme)
    .bind(page_size)
    .bind(compact_view)
    .bind(language)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)?;

    Ok(UserPreferences {
        theme: row.0,
        page_size: row.1,
        compact_view: row.2,
        language: row.3,
    })
}
