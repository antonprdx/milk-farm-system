use sqlx::PgPool;

use crate::errors::AppError;

#[derive(Debug, serde::Serialize)]
pub struct NotificationChannel {
    pub id: i32,
    pub user_id: i32,
    pub channel_type: String,
    pub channel_token: String,
    pub enabled: bool,
}

#[derive(Debug, serde::Deserialize)]
pub struct CreateChannel {
    pub channel_type: String,
    pub channel_token: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct CreateRule {
    pub event_type: String,
    pub channel_id: Option<i32>,
}

pub async fn list_channels(pool: &PgPool, user_id: i32) -> Result<Vec<NotificationChannel>, AppError> {
    sqlx::query_as::<_, (i32, i32, String, String, bool)>(
        "SELECT id, user_id, channel_type, channel_token, enabled FROM notification_channels WHERE user_id = $1 ORDER BY id",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)
    .map(|rows| rows.into_iter().map(|(id, user_id, channel_type, channel_token, enabled)| NotificationChannel { id, user_id, channel_type, channel_token, enabled }).collect())
}

pub async fn create_channel(pool: &PgPool, user_id: i32, req: &CreateChannel) -> Result<NotificationChannel, AppError> {
    if !matches!(req.channel_type.as_str(), "browser" | "telegram") {
        return Err(AppError::BadRequest("channel_type must be 'browser' or 'telegram'".into()));
    }
    let row = sqlx::query_as::<_, (i32, i32, String, String, bool)>(
        "INSERT INTO notification_channels (user_id, channel_type, channel_token) VALUES ($1, $2, $3)
         ON CONFLICT (user_id, channel_type) DO UPDATE SET channel_token = $3, enabled = true
         RETURNING id, user_id, channel_type, channel_token, enabled",
    )
    .bind(user_id)
    .bind(&req.channel_type)
    .bind(&req.channel_token)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)?;

    Ok(NotificationChannel { id: row.0, user_id: row.1, channel_type: row.2, channel_token: row.3, enabled: row.4 })
}

pub async fn delete_channel(pool: &PgPool, user_id: i32, id: i32) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM notification_channels WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Channel not found".into()));
    }
    Ok(())
}

pub async fn list_rules(pool: &PgPool, user_id: i32) -> Result<Vec<serde_json::Value>, AppError> {
    let rows: Vec<(i32, i32, String, Option<i32>, bool)> = sqlx::query_as(
        "SELECT id, user_id, event_type, channel_id, enabled FROM notification_rules WHERE user_id = $1 ORDER BY id",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    Ok(rows.into_iter().map(|(id, uid, et, cid, en)| serde_json::json!({ "id": id, "user_id": uid, "event_type": et, "channel_id": cid, "enabled": en })).collect())
}

pub async fn create_rule(pool: &PgPool, user_id: i32, req: &CreateRule) -> Result<(), AppError> {
    sqlx::query("INSERT INTO notification_rules (user_id, event_type, channel_id) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(&req.event_type)
        .bind(req.channel_id)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;
    Ok(())
}

pub async fn delete_rule(pool: &PgPool, user_id: i32, id: i32) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM notification_rules WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Rule not found".into()));
    }
    Ok(())
}

pub async fn send_telegram(bot_token: &str, chat_id: &str, message: &str) -> Result<(), AppError> {
    let url = format!("https://api.telegram.org/bot{}/sendMessage", bot_token);
    let client = reqwest::Client::new();
    client.post(&url)
        .json(&serde_json::json!({ "chat_id": chat_id, "text": message, "parse_mode": "HTML" }))
        .send()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Telegram send failed: {}", e)))?;
    Ok(())
}

pub async fn dispatch_alert(pool: &PgPool, alert_category: &str, message: &str) {
    let bot_token = crate::services::system_settings_service::get_value(pool, "telegram_bot_token")
        .await
        .unwrap_or_default();

    if bot_token.is_empty() {
        return;
    }

    let channels: Vec<(i32, String)> = sqlx::query_as(
        "SELECT DISTINCT nc.user_id, nc.channel_token
         FROM notification_channels nc
         JOIN notification_rules nr ON nr.channel_id = nc.id
         WHERE nc.channel_type = 'telegram' AND nc.enabled = true AND nr.enabled = true
           AND (nr.event_type = $1 OR nr.event_type = 'all')",
    )
    .bind(alert_category)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    for (_user_id, chat_id) in channels {
        if let Err(e) = send_telegram(&bot_token, &chat_id, message).await {
            tracing::warn!(error = %e, "Failed to send Telegram notification to {}", chat_id);
        }
    }
}
