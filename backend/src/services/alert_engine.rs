use std::sync::Arc;

use sqlx::PgPool;

use crate::errors::AppError;
use crate::handlers::events::EventBus;
use crate::models::alerts::*;
use crate::models::pagination::Pagination;
use crate::services::system_settings_service;

pub async fn run_alert_loop(pool: PgPool, event_bus: Arc<EventBus>) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(900));
    interval.tick().await;

    loop {
        interval.tick().await;
        match run_all_checks(&pool, &event_bus).await {
            Ok(count) => {
                if count > 0 {
                    tracing::info!(count, "Alert engine: created new alerts");
                }
            }
            Err(e) => {
                tracing::error!(error = %e, "Alert engine error");
            }
        }

        if let Err(e) = resolve_stale_alerts(&pool).await {
            tracing::error!(error = %e, "Alert engine: failed to resolve stale alerts");
        }
    }
}

async fn run_all_checks(pool: &PgPool, event_bus: &EventBus) -> Result<usize, AppError> {
    let thresholds = system_settings_service::get_alert_thresholds(pool)
        .await
        .map(Thresholds::from)
        .unwrap_or_else(|_| Thresholds::default());

    let mut total = 0;
    total += check_milk_drop(pool, event_bus, &thresholds).await?;
    total += check_high_scc(pool, event_bus, &thresholds).await?;
    total += check_activity_drop(pool, event_bus, &thresholds).await?;
    total += check_no_milking(pool, event_bus).await?;
    total += check_expected_calving(pool, event_bus, &thresholds).await?;
    total += check_low_feed(pool, event_bus).await?;
    Ok(total)
}

struct Thresholds {
    alert_min_milk: f64,
    alert_max_scc: f64,
    alert_days_before_calving: i32,
    alert_activity_drop_pct: i32,
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            alert_min_milk: 5.0,
            alert_max_scc: 400.0,
            alert_days_before_calving: 14,
            alert_activity_drop_pct: 30,
        }
    }
}

impl From<crate::models::system_settings::AlertThresholds> for Thresholds {
    fn from(t: crate::models::system_settings::AlertThresholds) -> Self {
        Thresholds {
            alert_min_milk: t.alert_min_milk,
            alert_max_scc: t.alert_max_scc,
            alert_days_before_calving: t.alert_days_before_calving,
            alert_activity_drop_pct: t.alert_activity_drop_pct,
        }
    }
}

async fn upsert_alert(
    pool: &PgPool,
    event_bus: &EventBus,
    category: &str,
    severity: &str,
    animal_id: Option<i32>,
    message: &str,
    details: serde_json::Value,
) -> Result<bool, AppError> {
    let row = sqlx::query_as::<_, (i32,)>(
        "SELECT id FROM alerts WHERE category = $1::alert_category AND animal_id IS NOT DISTINCT FROM $2 AND status = 'active'",
    )
    .bind(category)
    .bind(animal_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?;

    if let Some((alert_id,)) = row {
        sqlx::query(
            "UPDATE alerts SET message = $2, details = $3, severity = $4::alert_severity, detected_at = NOW() WHERE id = $1",
        )
        .bind(alert_id)
        .bind(message)
        .bind(details)
        .bind(severity)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;
        return Ok(false);
    }

    sqlx::query(
        "INSERT INTO alerts (category, severity, animal_id, message, details)
         VALUES ($1::alert_category, $2::alert_severity, $3, $4, $5)",
    )
    .bind(category)
    .bind(severity)
    .bind(animal_id)
    .bind(message)
    .bind(details)
    .execute(pool)
    .await
    .map_err(AppError::Database)?;

    let _ = event_bus.publish("alert", serde_json::json!({
        "category": category,
        "severity": severity,
        "animal_id": animal_id,
        "message": message,
    }));

    let msg = format!("[{}] {}", severity.to_uppercase(), message);
    let pool_c = pool.clone();
    let cat = category.to_string();
    tokio::spawn(async move {
        crate::services::notification_service::dispatch_alert(&pool_c, &cat, &msg).await;
    });

    Ok(true)
}

async fn check_milk_drop(
    pool: &PgPool,
    event_bus: &EventBus,
    thresholds: &Thresholds,
) -> Result<usize, AppError> {
    let factor = 1.0 - (thresholds.alert_min_milk / 100.0).min(0.99);

    let rows: Vec<(i32, Option<String>, f64, f64)> = sqlx::query_as(
        "SELECT a.id, a.name, short_avg.milk, long_avg.milk
         FROM animals a
         JOIN LATERAL (SELECT AVG(m.milk_amount)::float8 as milk FROM milk_day_productions m WHERE m.animal_id = a.id AND m.date >= get_ref_date() - INTERVAL '2 days') short_avg ON true
         JOIN LATERAL (SELECT AVG(m.milk_amount)::float8 as milk FROM milk_day_productions m WHERE m.animal_id = a.id AND m.date >= get_ref_date() - INTERVAL '14 days' AND m.date < get_ref_date() - INTERVAL '2 days') long_avg ON true
         WHERE a.active = true AND short_avg.milk IS NOT NULL AND long_avg.milk IS NOT NULL AND short_avg.milk < long_avg.milk * $1",
    )
    .bind(factor)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    let mut count = 0;
    for (id, _name, short_m, long_m) in rows {
        let pct = ((1.0 - short_m / long_m) * 100.0).round() as i32;
        let severity = if pct >= 40 { "critical" } else { "warning" };
        let created = upsert_alert(
            pool,
            event_bus,
            "milk_drop",
            severity,
            Some(id),
            &format!("Надой упал на {}% за последние 2 дня", pct),
            serde_json::json!({ "short_avg": short_m, "long_avg": long_m, "drop_pct": pct }),
        )
        .await?;
        if created {
            count += 1;
        }
    }
    Ok(count)
}

async fn check_high_scc(
    pool: &PgPool,
    event_bus: &EventBus,
    thresholds: &Thresholds,
) -> Result<usize, AppError> {
    let scc_threshold = thresholds.alert_max_scc.max(100.0);

    let rows: Vec<(i32, Option<String>, f64)> = sqlx::query_as(
        "SELECT a.id, a.name, recent.scc::float8
         FROM animals a
         JOIN LATERAL (SELECT q.scc FROM milk_quality q WHERE q.animal_id = a.id ORDER BY q.date DESC LIMIT 1) recent ON true
         WHERE a.active = true AND recent.scc IS NOT NULL AND recent.scc > $1",
    )
    .bind(scc_threshold)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    let mut count = 0;
    for (id, _name, scc) in rows {
        let severity = if scc > 500000.0 { "critical" } else { "warning" };
        let created = upsert_alert(
            pool,
            event_bus,
            "high_scc",
            severity,
            Some(id),
            &format!("SCC={:.0} превышает порог {:.0} — возможен мастит", scc, scc_threshold),
            serde_json::json!({ "scc": scc, "threshold": scc_threshold }),
        )
        .await?;
        if created {
            count += 1;
        }
    }
    Ok(count)
}

async fn check_activity_drop(
    pool: &PgPool,
    event_bus: &EventBus,
    thresholds: &Thresholds,
) -> Result<usize, AppError> {
    let drop_pct = thresholds.alert_activity_drop_pct.min(99) as f64;
    let factor = 1.0 - (drop_pct / 100.0);

    let rows: Vec<(i32, Option<String>, f64, f64)> = sqlx::query_as(
        "SELECT a.id, a.name, recent.act, baseline.avg_act
         FROM animals a
         JOIN LATERAL (
             SELECT AVG(activity_counter)::float8 as act FROM activities
             WHERE animal_id = a.id AND activity_datetime >= get_ref_date() - INTERVAL '1 day'
         ) recent ON true
         JOIN LATERAL (
             SELECT AVG(activity_counter)::float8 as avg_act FROM activities
             WHERE animal_id = a.id AND activity_datetime >= get_ref_date() - INTERVAL '14 days'
         ) baseline ON true
         WHERE a.active = true AND recent.act IS NOT NULL AND baseline.avg_act IS NOT NULL AND recent.act < baseline.avg_act * $1",
    )
    .bind(factor)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    let mut count = 0;
    for (id, _name, recent, baseline) in rows {
        let pct = ((1.0 - recent / baseline) * 100.0).round() as i32;
        let created = upsert_alert(
            pool,
            event_bus,
            "activity_drop",
            "warning",
            Some(id),
            &format!("Активность снизилась на {}%", pct),
            serde_json::json!({ "recent": recent, "baseline": baseline, "drop_pct": pct }),
        )
        .await?;
        if created {
            count += 1;
        }
    }
    Ok(count)
}

async fn check_no_milking(pool: &PgPool, event_bus: &EventBus) -> Result<usize, AppError> {
    let rows: Vec<(i32, Option<String>)> = sqlx::query_as(
        "SELECT a.id, a.name FROM animals a
         WHERE a.active = true AND a.gender = 'female'
         AND NOT EXISTS (
             SELECT 1 FROM milk_day_productions m WHERE m.animal_id = a.id AND m.date >= get_ref_date() - INTERVAL '1 day'
         )
         AND EXISTS (
             SELECT 1 FROM milk_day_productions m WHERE m.animal_id = a.id AND m.date >= get_ref_date() - INTERVAL '7 days'
         )
         AND NOT EXISTS (
             SELECT 1 FROM dry_offs d WHERE d.animal_id = a.id AND d.date <= get_ref_date()
         )",
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    let mut count = 0;
    for (id, _name) in rows {
        let created = upsert_alert(
            pool,
            event_bus,
            "no_milking",
            "warning",
            Some(id),
            "Корова не доилась более 24 часов",
            serde_json::json!({}),
        )
        .await?;
        if created {
            count += 1;
        }
    }
    Ok(count)
}

async fn check_expected_calving(
    pool: &PgPool,
    event_bus: &EventBus,
    thresholds: &Thresholds,
) -> Result<usize, AppError> {
    let days = thresholds.alert_days_before_calving;

    let rows: Vec<(i32, Option<String>)> = sqlx::query_as(
        "SELECT a.id, a.name FROM animals a
         JOIN calvings c ON c.animal_id = a.id
         JOIN inseminations i ON i.animal_id = a.id AND i.date > c.date
         LEFT JOIN pregnancies p ON p.animal_id = a.id AND p.insemination_id = i.id
         LEFT JOIN calvings c2 ON c2.animal_id = a.id AND c2.date > c.date
         WHERE a.active = true
           AND p.id IS NOT NULL
           AND c2.id IS NULL
           AND (i.date + INTERVAL '283 days') BETWEEN get_ref_date() AND get_ref_date() + $1::int * INTERVAL '1 day'",
    )
    .bind(days)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    let mut count = 0;
    for (id, _name) in rows {
        let created = upsert_alert(
            pool,
            event_bus,
            "expected_calving",
            "info",
            Some(id),
            &format!("Ожидается отёл в течение {} дней", days),
            serde_json::json!({ "days_before_calving": days }),
        )
        .await?;
        if created {
            count += 1;
        }
    }
    Ok(count)
}

async fn check_low_feed(pool: &PgPool, event_bus: &EventBus) -> Result<usize, AppError> {
    let rows: Vec<(i32, Option<String>, f64, f64)> = sqlx::query_as(
        "SELECT a.id, a.name, recent.avg_feed, baseline.avg_feed as baseline_feed
         FROM animals a
         JOIN LATERAL (
             SELECT AVG(total)::float8 as avg_feed FROM feed_day_amounts
             WHERE animal_id = a.id AND feed_date >= get_ref_date() - INTERVAL '2 days'
         ) recent ON true
         JOIN LATERAL (
             SELECT AVG(total)::float8 as avg_feed FROM feed_day_amounts
             WHERE animal_id = a.id AND feed_date >= get_ref_date() - INTERVAL '14 days'
         ) baseline ON true
         WHERE a.active = true AND recent.avg_feed IS NOT NULL AND baseline.avg_feed IS NOT NULL
           AND recent.avg_feed < baseline.avg_feed * 0.8",
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    let mut count = 0;
    for (id, _name, recent, baseline) in rows {
        let pct = ((1.0 - recent / baseline) * 100.0).round() as i32;
        let created = upsert_alert(
            pool,
            event_bus,
            "low_feed",
            "warning",
            Some(id),
            &format!("Потребление корма снизилось на {}%", pct),
            serde_json::json!({ "recent": recent, "baseline": baseline, "drop_pct": pct }),
        )
        .await?;
        if created {
            count += 1;
        }
    }
    Ok(count)
}

async fn resolve_stale_alerts(pool: &PgPool) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE alerts SET status = 'resolved', resolved_at = NOW()
         WHERE status = 'active' AND category = 'milk_drop'
           AND animal_id IS NOT NULL
           AND NOT EXISTS (
               SELECT 1 FROM milk_day_productions m
               WHERE m.animal_id = alerts.animal_id
               AND m.date >= get_ref_date() - INTERVAL '2 days'
               AND m.milk_amount < (
                   SELECT AVG(m2.milk_amount) FROM milk_day_productions m2
                   WHERE m2.animal_id = alerts.animal_id
                   AND m2.date >= get_ref_date() - INTERVAL '14 days'
                   AND m2.date < get_ref_date() - INTERVAL '2 days'
               ) * (1.0 - (SELECT value::float8 FROM system_settings WHERE key = 'alert_min_milk') / 100.0)
           )",
    )
    .execute(pool)
    .await
    .map_err(AppError::Database)?;

    sqlx::query(
        "UPDATE alerts SET status = 'resolved', resolved_at = NOW()
         WHERE status = 'active' AND category = 'no_milking'
           AND animal_id IS NOT NULL
           AND EXISTS (
               SELECT 1 FROM milk_day_productions m
               WHERE m.animal_id = alerts.animal_id AND m.date >= get_ref_date() - INTERVAL '1 day'
           )",
    )
    .execute(pool)
    .await
    .map_err(AppError::Database)?;

    sqlx::query(
        "UPDATE alerts SET status = 'resolved', resolved_at = NOW()
         WHERE status = 'active' AND category = 'expected_calving'
           AND animal_id IS NOT NULL
           AND EXISTS (
               SELECT 1 FROM calvings c WHERE c.animal_id = alerts.animal_id AND c.date >= get_ref_date() - INTERVAL '3 days'
           )",
    )
    .execute(pool)
    .await
    .map_err(AppError::Database)?;

    Ok(())
}

pub async fn list_alerts(
    pool: &PgPool,
    filter: &AlertFilter,
) -> Result<AlertsListResponse, AppError> {
    let pag = Pagination::from_filter(filter.page, filter.per_page);

    let mut where_parts: Vec<String> = Vec::new();
    let mut status_val: Option<String> = None;
    let mut category_val: Option<String> = None;
    let mut severity_val: Option<String> = None;
    let mut animal_id_val: Option<i32> = None;

    let mut next_param = 1usize;

    if let Some(ref s) = filter.status {
        where_parts.push(format!("a.status = ${}", next_param));
        status_val = Some(s.clone());
        next_param += 1;
    }
    if let Some(ref c) = filter.category {
        where_parts.push(format!("a.category = ${}", next_param));
        category_val = Some(c.clone());
        next_param += 1;
    }
    if let Some(ref s) = filter.severity {
        where_parts.push(format!("a.severity = ${}", next_param));
        severity_val = Some(s.clone());
        next_param += 1;
    }
    if let Some(id) = filter.animal_id {
        where_parts.push(format!("a.animal_id = ${}", next_param));
        animal_id_val = Some(id);
        next_param += 1;
    }

    let where_sql = if where_parts.is_empty() {
        "1=1".to_string()
    } else {
        where_parts.join(" AND ")
    };

    let count_sql = format!("SELECT COUNT(*) FROM alerts a WHERE {}", where_sql);
    let mut count_q = sqlx::query_as::<_, (i64,)>(&count_sql);
    if let Some(ref v) = status_val { count_q = count_q.bind(v); }
    if let Some(ref v) = category_val { count_q = count_q.bind(v); }
    if let Some(ref v) = severity_val { count_q = count_q.bind(v); }
    if let Some(v) = animal_id_val { count_q = count_q.bind(v); }
    let total = count_q.fetch_one(pool).await.map_err(AppError::Database)?;

    let limit_param = next_param;
    let offset_param = next_param + 1;
    let data_sql = format!(
        "SELECT a.id, a.category::text, a.severity::text, a.status::text,
                a.animal_id, a.message, a.details,
                TO_CHAR(a.detected_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"'),
                TO_CHAR(a.acknowledged_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"'),
                TO_CHAR(a.resolved_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"'),
                an.name, an.life_number
         FROM alerts a
         LEFT JOIN animals an ON an.id = a.animal_id
         WHERE {}
         ORDER BY
             CASE a.severity WHEN 'critical' THEN 0 WHEN 'warning' THEN 1 WHEN 'info' THEN 2 END,
             a.detected_at DESC
         LIMIT ${} OFFSET ${}",
        where_sql, limit_param, offset_param
    );

    let mut data_q = sqlx::query_as::<_, (i32, String, String, String, Option<i32>, String, Option<serde_json::Value>, String, Option<String>, Option<String>, Option<String>, Option<String>)>(&data_sql);
    if let Some(ref v) = status_val { data_q = data_q.bind(v); }
    if let Some(ref v) = category_val { data_q = data_q.bind(v); }
    if let Some(ref v) = severity_val { data_q = data_q.bind(v); }
    if let Some(v) = animal_id_val { data_q = data_q.bind(v); }
    data_q = data_q.bind(pag.per_page).bind(pag.offset);
    let rows = data_q.fetch_all(pool).await.map_err(AppError::Database)?;

    let data = rows
        .into_iter()
        .map(|(id, cat, sev, status, animal_id, message, details, detected_at, ack_at, res_at, animal_name, life_number)| {
            AlertRecord {
                id,
                category: parse_category(&cat),
                severity: parse_severity(&sev),
                status: parse_status(&status),
                animal_id,
                message,
                details,
                detected_at,
                acknowledged_at: ack_at,
                resolved_at: res_at,
                animal_name,
                life_number,
            }
        })
        .collect();

    Ok(AlertsListResponse {
        data,
        total: total.0,
        page: pag.page,
        per_page: pag.per_page,
    })
}

pub async fn get_active_summary(pool: &PgPool) -> Result<ActiveAlertsSummary, AppError> {
    let (total, critical, warning, info): (i64, i64, i64, i64) = sqlx::query_as(
        "SELECT COUNT(*),
                COUNT(*) FILTER (WHERE severity = 'critical'),
                COUNT(*) FILTER (WHERE severity = 'warning'),
                COUNT(*) FILTER (WHERE severity = 'info')
         FROM alerts WHERE status = 'active'",
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)?;

    let categories: Vec<(String, i64)> = sqlx::query_as(
        "SELECT category::text, COUNT(*) FROM alerts WHERE status = 'active' GROUP BY category",
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    let by_category: serde_json::Map<String, serde_json::Value> = categories
        .into_iter()
        .map(|(k, v)| (k, serde_json::Value::Number(v.into())))
        .collect();

    Ok(ActiveAlertsSummary {
        total_active: total,
        critical_count: critical,
        warning_count: warning,
        info_count: info,
        by_category: serde_json::Value::Object(by_category),
    })
}

pub async fn acknowledge_alert(pool: &PgPool, id: i32) -> Result<(), AppError> {
    let result = sqlx::query(
        "UPDATE alerts SET status = 'acknowledged', acknowledged_at = NOW() WHERE id = $1 AND status = 'active'",
    )
    .bind(id)
    .execute(pool)
    .await
    .map_err(AppError::Database)?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Alert {} not found or not active", id)));
    }
    Ok(())
}

pub async fn acknowledge_all(pool: &PgPool) -> Result<u64, AppError> {
    let result = sqlx::query(
        "UPDATE alerts SET status = 'acknowledged', acknowledged_at = NOW() WHERE status = 'active'",
    )
    .execute(pool)
    .await
    .map_err(AppError::Database)?;
    Ok(result.rows_affected())
}

fn parse_category(s: &str) -> AlertCategory {
    match s {
        "milk_drop" => AlertCategory::MilkDrop,
        "high_scc" => AlertCategory::HighScc,
        "activity_drop" => AlertCategory::ActivityDrop,
        "low_feed" => AlertCategory::LowFeed,
        "no_milking" => AlertCategory::NoMilking,
        "ketosis_risk" => AlertCategory::KetosisRisk,
        "mastitis_risk" => AlertCategory::MastitisRisk,
        "expected_calving" => AlertCategory::ExpectedCalving,
        "equipment_anomaly" => AlertCategory::EquipmentAnomaly,
        _ => AlertCategory::Other,
    }
}

fn parse_severity(s: &str) -> AlertSeverity {
    match s {
        "critical" => AlertSeverity::Critical,
        "warning" => AlertSeverity::Warning,
        _ => AlertSeverity::Info,
    }
}

fn parse_status(s: &str) -> AlertStatus {
    match s {
        "active" => AlertStatus::Active,
        "acknowledged" => AlertStatus::Acknowledged,
        "resolved" => AlertStatus::Resolved,
        _ => AlertStatus::Active,
    }
}
