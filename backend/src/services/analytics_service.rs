use sqlx::PgPool;

use crate::errors::AppError;
use crate::models::analytics::*;
use crate::models::system_settings::AlertThresholds;
use crate::services::system_settings_service;

pub async fn kpi(pool: &PgPool) -> Result<KpiResponse, AppError> {
    let (
        avg_calving_interval_days,
        conception_rate_pct,
        avg_milk_by_lactation,
        feed_efficiency,
        avg_days_to_first_ai,
        avg_scc,
        refusal_rate_pct,
    ) = tokio::try_join!(
        avg_calving_interval(pool),
        conception_rate(pool),
        milk_by_lactation(pool),
        feed_eff(pool),
        days_to_first_ai(pool),
        avg_scc_val(pool),
        refusal_rate(pool),
    )?;

    let culling_risk = culling_risk_calc(pool).await?;

    Ok(KpiResponse {
        avg_calving_interval_days,
        conception_rate_pct,
        avg_milk_by_lactation,
        feed_efficiency,
        avg_days_to_first_ai,
        avg_scc,
        refusal_rate_pct,
        culling_risk,
    })
}

async fn avg_calving_interval(pool: &PgPool) -> Result<Option<f64>, AppError> {
    let row: Option<(Option<f64>,)> = sqlx::query_as(
        "SELECT AVG(c2.calving_date - c1.calving_date)::float8
         FROM calvings c1
         JOIN calvings c2 ON c1.animal_id = c2.animal_id AND c2.calving_date > c1.calving_date
         WHERE NOT EXISTS (
             SELECT 1 FROM calvings c3
             WHERE c3.animal_id = c1.animal_id
             AND c3.calving_date > c1.calving_date AND c3.calving_date < c2.calving_date
         )",
    )
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?;
    Ok(row.and_then(|r| r.0.filter(|&v| v > 0.0)))
}

async fn conception_rate(pool: &PgPool) -> Result<Option<f64>, AppError> {
    let row: Option<(i64, i64)> = sqlx::query_as(
        "SELECT
            (SELECT COUNT(*) FROM inseminations WHERE insemination_date >= CURRENT_DATE - INTERVAL '12 months')::int8 as ins,
            (SELECT COUNT(*) FROM pregnancies WHERE pregnancy_date >= CURRENT_DATE - INTERVAL '12 months')::int8 as preg",
    )
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?;

    match row {
        Some((ins, preg)) if ins > 0 => Ok(Some((preg as f64 / ins as f64) * 100.0)),
        _ => Ok(None),
    }
}

async fn milk_by_lactation(pool: &PgPool) -> Result<Vec<LactationAvg>, AppError> {
    let rows: Vec<(i32, Option<f64>)> = sqlx::query_as(
        "SELECT c.lac_number, AVG(m.milk_amount)::float8
         FROM milk_day_productions m
         JOIN calvings c ON c.animal_id = m.animal_id
         WHERE c.lac_number IS NOT NULL
           AND m.date >= c.calving_date
           AND m.date < c.calving_date + INTERVAL '400 days'
         GROUP BY c.lac_number
         ORDER BY c.lac_number",
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    Ok(rows
        .into_iter()
        .map(|(lac, avg)| LactationAvg { lac, avg_milk: avg })
        .collect())
}

async fn feed_eff(pool: &PgPool) -> Result<Option<f64>, AppError> {
    let row: Option<(Option<f64>, Option<f64>)> = sqlx::query_as(
        "SELECT
            (SELECT SUM(milk_amount)::float8 FROM milk_day_productions WHERE date >= CURRENT_DATE - INTERVAL '30 days') as milk,
            (SELECT SUM(total)::float8 FROM feed_day_amounts WHERE feed_date >= CURRENT_DATE - INTERVAL '30 days') as feed",
    )
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?;

    match row {
        Some((Some(milk), Some(feed))) if feed > 0.0 => Ok(Some(milk / feed)),
        _ => Ok(None),
    }
}

async fn days_to_first_ai(pool: &PgPool) -> Result<Option<f64>, AppError> {
    let row: Option<(Option<f64>,)> = sqlx::query_as(
        "SELECT AVG(first_ai - calving_date)::float8 FROM (
            SELECT c.animal_id, c.calving_date,
                   (SELECT MIN(i.insemination_date) FROM inseminations i
                    WHERE i.animal_id = c.animal_id AND i.insemination_date > c.calving_date) as first_ai
            FROM calvings c
            WHERE c.calving_date >= CURRENT_DATE - INTERVAL '24 months'
        ) sub WHERE first_ai IS NOT NULL",
    )
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?;
    Ok(row.and_then(|r| r.0.filter(|&v| v > 0.0)))
}

async fn avg_scc_val(pool: &PgPool) -> Result<Option<f64>, AppError> {
    let row: Option<(Option<f64>,)> = sqlx::query_as(
        "SELECT AVG(scc)::float8 FROM milk_quality WHERE date >= CURRENT_DATE - INTERVAL '90 days'",
    )
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?;
    Ok(row.and_then(|r| r.0.filter(|&v| v > 0.0)))
}

async fn refusal_rate(pool: &PgPool) -> Result<Option<f64>, AppError> {
    let row: Option<(Option<f64>, Option<f64>)> = sqlx::query_as(
        "SELECT
            SUM(refusals)::float8 as refusals,
            SUM(milkings)::float8 as milkings
         FROM milk_quality WHERE date >= CURRENT_DATE - INTERVAL '90 days'",
    )
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?;

    match row {
        Some((Some(refusals), Some(milkings))) if milkings > 0.0 => {
            Ok(Some((refusals / milkings) * 100.0))
        }
        _ => Ok(None),
    }
}

#[allow(clippy::type_complexity)]
async fn culling_risk_calc(pool: &PgPool) -> Result<Vec<CullingRiskEntry>, AppError> {
    let rows: Vec<(i32, Option<String>, Option<String>, Option<f64>, Option<f64>, Option<f64>, Option<i64>)> = sqlx::query_as(
        "SELECT a.id, a.name, a.life_number,
                latest_milk.milk as recent_milk,
                latest_scc.scc as recent_scc,
                ci.avg_interval as avg_interval,
                EXTRACT(YEAR FROM AGE(CURRENT_DATE, a.birth_date))::int8 as age_years
         FROM animals a
         LEFT JOIN LATERAL (
             SELECT AVG(m.milk_amount)::float8 as milk
             FROM milk_day_productions m WHERE m.animal_id = a.id AND m.date >= CURRENT_DATE - INTERVAL '30 days'
         ) latest_milk ON true
         LEFT JOIN LATERAL (
             SELECT AVG(q.scc)::float8 as scc
             FROM milk_quality q WHERE q.animal_id = a.id AND q.date >= CURRENT_DATE - INTERVAL '90 days'
         ) latest_scc ON true
         LEFT JOIN LATERAL (
              SELECT AVG(c2.calving_date - c1.calving_date)::float8 as avg_interval
             FROM calvings c1
             JOIN calvings c2 ON c1.animal_id = c2.animal_id AND c2.calving_date > c1.calving_date
             WHERE c1.animal_id = a.id
             AND NOT EXISTS (SELECT 1 FROM calvings c3 WHERE c3.animal_id = c1.animal_id AND c3.calving_date > c1.calving_date AND c3.calving_date < c2.calving_date)
         ) ci ON true
         WHERE a.active = true AND a.gender = 'female'
         ORDER BY a.id",
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    let mut entries: Vec<CullingRiskEntry> = rows
        .into_iter()
        .map(|(id, name, life_number, milk, scc, interval, age)| {
            let mut score = 0.0_f64;
            let mut reasons = Vec::new();

            if let Some(age_yrs) = age {
                if age_yrs >= 8 {
                    score += 0.3;
                    reasons.push("возраст".to_string());
                } else if age_yrs >= 6 {
                    score += 0.15;
                    reasons.push("пожилой возраст".to_string());
                }
            }

            if let Some(milk) = milk {
                if milk < 15.0 {
                    score += 0.3;
                    reasons.push("низкий надой".to_string());
                } else if milk < 20.0 {
                    score += 0.1;
                    reasons.push("снижающийся надой".to_string());
                }
            }

            if let Some(scc_val) = scc {
                if scc_val > 300000.0 {
                    score += 0.25;
                    reasons.push("высокий SCC".to_string());
                } else if scc_val > 200000.0 {
                    score += 0.1;
                    reasons.push("повышенный SCC".to_string());
                }
            }

            if let Some(ci) = interval {
                if ci > 450.0 {
                    score += 0.2;
                    reasons.push("длинный интервал отёлов".to_string());
                } else if ci > 400.0 {
                    score += 0.1;
                    reasons.push("увеличенный интервал отёлов".to_string());
                }
            }

            CullingRiskEntry {
                animal_id: id,
                name,
                life_number,
                score: (score * 100.0).round() / 100.0,
                reasons,
            }
        })
        .filter(|e| e.score >= 0.3)
        .collect();

    entries.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    entries.truncate(10);
    Ok(entries)
}

pub async fn alerts(pool: &PgPool) -> Result<AlertsResponse, AppError> {
    let thresholds = system_settings_service::get_alert_thresholds(pool)
        .await
        .unwrap_or(AlertThresholds {
            alert_min_milk: 5.0,
            alert_max_scc: 400.0,
            alert_days_before_calving: 14,
            alert_activity_drop_pct: 30,
        });

    let milk_drop_factor = 1.0 - (thresholds.alert_min_milk / 100.0).min(0.99);
    let scc_multiplier = if thresholds.alert_max_scc > 0.0 {
        thresholds.alert_max_scc / 200.0
    } else {
        2.0
    };
    let activity_drop_factor = 1.0 - (thresholds.alert_activity_drop_pct as f64 / 100.0).min(0.99);

    let mut alerts_list = Vec::new();

    let milk_drops: Vec<(i32, Option<String>, f64, f64)> = sqlx::query_as(
        "SELECT a.id, a.name, short_avg.milk, long_avg.milk
         FROM animals a
         JOIN LATERAL (SELECT AVG(m.milk_amount)::float8 as milk FROM milk_day_productions m WHERE m.animal_id = a.id AND m.date >= CURRENT_DATE - INTERVAL '7 days') short_avg ON true
         JOIN LATERAL (SELECT AVG(m.milk_amount)::float8 as milk FROM milk_day_productions m WHERE m.animal_id = a.id AND m.date >= CURRENT_DATE - INTERVAL '30 days' AND m.date < CURRENT_DATE - INTERVAL '7 days') long_avg ON true
         WHERE short_avg.milk IS NOT NULL AND long_avg.milk IS NOT NULL AND short_avg.milk < long_avg.milk * $1
         LIMIT 10",
    )
    .bind(milk_drop_factor)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    for (id, name, short_m, long_m) in milk_drops {
        let pct = ((1.0 - short_m / long_m) * 100.0).round() as i32;
        alerts_list.push(Alert {
            alert_type: "milk_drop".to_string(),
            severity: "warning".to_string(),
            animal_id: Some(id),
            animal_name: name,
            message: format!("Надой упал на {}% за последние 7 дней", pct),
            value: format!("{:.1} л → {:.1} л", long_m, short_m),
        });
    }

    let scc_spikes: Vec<(i32, Option<String>, f64, f64)> = sqlx::query_as(
        "SELECT a.id, a.name, recent.scc::float8, baseline.avg_scc
         FROM animals a
         JOIN LATERAL (SELECT q.scc FROM milk_quality q WHERE q.animal_id = a.id ORDER BY q.date DESC LIMIT 1) recent ON true
         JOIN LATERAL (SELECT AVG(q.scc)::float8 as avg_scc FROM milk_quality q WHERE q.animal_id = a.id AND q.date >= CURRENT_DATE - INTERVAL '90 days') baseline ON true
         WHERE recent.scc IS NOT NULL AND baseline.avg_scc IS NOT NULL AND recent.scc > baseline.avg_scc * $1
         LIMIT 10",
    )
    .bind(scc_multiplier)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    for (id, name, scc, avg) in scc_spikes {
        alerts_list.push(Alert {
            alert_type: "high_scc".to_string(),
            severity: if scc > 500000.0 {
                "critical"
            } else {
                "warning"
            }
            .to_string(),
            animal_id: Some(id),
            animal_name: name,
            message: "SCC значительно выше нормы — возможен мастит".to_string(),
            value: format!("SCC: {:.0} (среднее: {:.0})", scc, avg),
        });
    }

    let activity_drops: Vec<(i32, Option<String>, f64, f64)> = sqlx::query_as(
        "SELECT a.id, a.name, recent.act, baseline.avg_act
         FROM animals a
         JOIN LATERAL (
             SELECT AVG(activity_counter)::float8 as act FROM activities
             WHERE animal_id = a.id AND activity_datetime >= CURRENT_DATE - INTERVAL '1 day'
         ) recent ON true
         JOIN LATERAL (
             SELECT AVG(activity_counter)::float8 as avg_act FROM activities
             WHERE animal_id = a.id AND activity_datetime >= CURRENT_DATE - INTERVAL '7 days' AND activity_datetime < CURRENT_DATE - INTERVAL '1 day'
         ) baseline ON true
         WHERE recent.act IS NOT NULL AND baseline.avg_act IS NOT NULL AND recent.act < baseline.avg_act * $1
         LIMIT 10",
    )
    .bind(activity_drop_factor)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    for (id, name, act, base) in activity_drops {
        alerts_list.push(Alert {
            alert_type: "activity_drop".to_string(),
            severity: "warning".to_string(),
            animal_id: Some(id),
            animal_name: name,
            message: format!(
                "Активность ниже нормы на {}%+",
                thresholds.alert_activity_drop_pct
            ),
            value: format!("{:.0} → {:.0}", base, act),
        });
    }

    let rum_drops: Vec<(i32, Option<String>, f64, f64)> = sqlx::query_as(
        "SELECT a.id, a.name, recent.rum, baseline.avg_rum
         FROM animals a
         JOIN LATERAL (
             SELECT rumination_minutes::float8 as rum FROM ruminations
             WHERE animal_id = a.id ORDER BY date DESC LIMIT 1
         ) recent ON true
         JOIN LATERAL (
             SELECT AVG(rumination_minutes)::float8 as avg_rum FROM ruminations
             WHERE animal_id = a.id AND date >= CURRENT_DATE - INTERVAL '7 days' AND date < CURRENT_DATE
         ) baseline ON true
         WHERE recent.rum IS NOT NULL AND baseline.avg_rum IS NOT NULL AND recent.rum < baseline.avg_rum * 0.75
         LIMIT 10",
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    for (id, name, rum, base) in rum_drops {
        alerts_list.push(Alert {
            alert_type: "rumination_drop".to_string(),
            severity: "info".to_string(),
            animal_id: Some(id),
            animal_name: name,
            message: "Жвачка снизилась — следите за коровой".to_string(),
            value: format!("{:.0} мин → {:.0} мин", base, rum),
        });
    }

    alerts_list.sort_by(|a, b| {
        let ord = |s: &str| {
            if s == "critical" {
                0
            } else if s == "warning" {
                1
            } else {
                2
            }
        };
        ord(&a.severity).cmp(&ord(&b.severity))
    });

    Ok(AlertsResponse {
        alerts: alerts_list,
    })
}

pub async fn milk_trend(
    pool: &PgPool,
    days: i64,
    forecast_days: i64,
) -> Result<MilkTrendResponse, AppError> {
    let daily: Vec<(String, Option<f64>, Option<i64>)> = sqlx::query_as(
        "SELECT date::text, SUM(milk_amount)::float8, COUNT(DISTINCT animal_id)::int8
         FROM milk_day_productions
         WHERE date >= CURRENT_DATE - ($1 || ' days')::interval
         GROUP BY date ORDER BY date",
    )
    .bind(days)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    let daily_pts: Vec<DailyMilkPoint> = daily
        .into_iter()
        .map(|(d, m, c)| DailyMilkPoint {
            date: d,
            total_milk: m,
            cow_count: c,
        })
        .collect();

    let values: Vec<f64> = daily_pts.iter().filter_map(|p| p.total_milk).collect();

    let (forecast, direction) = if values.len() >= 7 {
        let (level, trend) = holt_forecast(&values, 0.3, 0.1);
        let mut fc = Vec::new();
        let last_date = daily_pts
            .last()
            .and_then(|p| chrono::NaiveDate::parse_from_str(&p.date, "%Y-%m-%d").ok());
        if let Some(ld) = last_date {
            for h in 1..=forecast_days {
                let pred = level + h as f64 * trend;
                let d = ld + chrono::Duration::days(h);
                let err_margin = pred * 0.1 * (1.0 + h as f64 * 0.05);
                fc.push(ForecastPoint {
                    date: d.format("%Y-%m-%d").to_string(),
                    predicted: (pred * 100.0).round() / 100.0,
                    lower: ((pred - err_margin) * 100.0).round() / 100.0,
                    upper: ((pred + err_margin) * 100.0).round() / 100.0,
                });
            }
        }
        let dir = if trend > 1.0 {
            "up"
        } else if trend < -1.0 {
            "down"
        } else {
            "stable"
        };
        (fc, dir.to_string())
    } else {
        (Vec::new(), "insufficient_data".to_string())
    };

    Ok(MilkTrendResponse {
        daily: daily_pts,
        forecast,
        trend_direction: direction,
    })
}

fn holt_forecast(values: &[f64], alpha: f64, beta: f64) -> (f64, f64) {
    if values.is_empty() {
        return (0.0, 0.0);
    }
    let mut level = values[0];
    let mut trend = if values.len() > 1 {
        values[1] - values[0]
    } else {
        0.0
    };

    for val in values.iter().skip(1) {
        let new_level = alpha * val + (1.0 - alpha) * (level + trend);
        let new_trend = beta * (new_level - level) + (1.0 - beta) * trend;
        level = new_level;
        trend = new_trend;
    }

    (level, trend)
}

#[allow(clippy::type_complexity)]
pub async fn reproduction_forecast(
    pool: &PgPool,
) -> Result<ReproductionForecastResponse, AppError> {
    let expected_calvings: Vec<(i32, Option<String>, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT a.id, a.name, a.life_number, i.insemination_date::text
         FROM inseminations i
         JOIN animals a ON a.id = i.animal_id
         WHERE a.active = true AND a.gender = 'female'
         AND NOT EXISTS (
             SELECT 1 FROM calvings c WHERE c.animal_id = i.animal_id AND c.calving_date > i.insemination_date
         )
         AND NOT EXISTS (
             SELECT 1 FROM dry_offs d WHERE d.animal_id = i.animal_id AND d.dry_off_date > i.insemination_date
         )
         AND i.insemination_date >= CURRENT_DATE - INTERVAL '300 days'
         ORDER BY i.insemination_date DESC",
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    let today = chrono::Utc::now().date_naive();
    let calvings: Vec<ExpectedCalving> = expected_calvings
        .into_iter()
        .filter_map(|(id, name, ln, ins_date)| {
            let ins = chrono::NaiveDate::parse_from_str(ins_date.as_deref()?, "%Y-%m-%d").ok()?;
            let expected = ins + chrono::Duration::days(283);
            Some(ExpectedCalving {
                animal_id: id,
                name,
                life_number: ln,
                insemination_date: ins_date,
                expected_date: expected.format("%Y-%m-%d").to_string(),
                days_left: (expected - today).num_days(),
            })
        })
        .collect();

    let dry_offs: Vec<DryOffRecommendation> = calvings
        .iter()
        .filter_map(|c| {
            let exp = chrono::NaiveDate::parse_from_str(&c.expected_date, "%Y-%m-%d").ok()?;
            let rec = exp - chrono::Duration::days(60);
            let days_until = (rec - today).num_days();
            if days_until <= 60 {
                Some(DryOffRecommendation {
                    animal_id: c.animal_id,
                    name: c.name.clone(),
                    life_number: c.life_number.clone(),
                    expected_calving: c.expected_date.clone(),
                    recommended_dry_off: rec.format("%Y-%m-%d").to_string(),
                    days_until_dry_off: days_until,
                })
            } else {
                None
            }
        })
        .collect();

    let last_heats: Vec<(i32, Option<String>, Option<String>, String)> = sqlx::query_as(
        "SELECT DISTINCT ON (h.animal_id) a.id, a.name, a.life_number, h.heat_date::text
         FROM heats h JOIN animals a ON a.id = h.animal_id
         WHERE a.active = true AND a.gender = 'female'
         AND h.heat_date >= CURRENT_DATE - INTERVAL '45 days'
         ORDER BY h.animal_id, h.heat_date DESC",
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    let heats: Vec<ExpectedHeat> = last_heats
        .into_iter()
        .filter_map(|(id, name, ln, heat_str)| {
            let heat_date = chrono::NaiveDate::parse_from_str(&heat_str, "%Y-%m-%d").ok()?;
            let expected = heat_date + chrono::Duration::days(21);
            let days_until = (expected - today).num_days();
            Some(ExpectedHeat {
                animal_id: id,
                name,
                life_number: ln,
                last_heat: heat_str,
                expected_next: expected.format("%Y-%m-%d").to_string(),
                days_until,
                overdue: days_until < 0,
            })
        })
        .collect();

    Ok(ReproductionForecastResponse {
        expected_calvings: calvings,
        expected_heats: heats,
        dry_off_recommendations: dry_offs,
    })
}

pub async fn feed_forecast(pool: &PgPool) -> Result<FeedForecastResponse, AppError> {
    let row: Option<(Option<f64>, Option<f64>)> = sqlx::query_as(
        "SELECT
            (SELECT SUM(total)::float8 FROM feed_day_amounts WHERE feed_date >= CURRENT_DATE - INTERVAL '7 days') as weekly,
            (SELECT AVG(total)::float8 FROM feed_day_amounts WHERE feed_date >= CURRENT_DATE - INTERVAL '30 days') as avg_daily",
    )
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?;

    let (weekly_feed, avg_daily) = match row {
        Some((w, d)) => (w, d),
        None => (None, None),
    };

    let avg_per_cow_day = avg_daily;
    let predicted = avg_daily.unwrap_or(0.0) * 7.0;

    let milk_per_feed = feed_eff(pool).await?;

    Ok(FeedForecastResponse {
        weekly_feed_kg: weekly_feed,
        predicted_next_week_kg: (predicted * 100.0).round() / 100.0,
        avg_per_cow_day_kg: avg_per_cow_day,
        milk_per_feed,
    })
}

#[derive(Debug, sqlx::FromRow)]
struct LatestMilkRow {
    animal_id: i32,
    name: Option<String>,
    date: String,
    milk_amount: Option<f64>,
    avg_amount: Option<f64>,
    isk: Option<f64>,
}

pub async fn latest_milk(pool: &PgPool) -> Result<Vec<LatestMilkEntry>, AppError> {
    let rows: Vec<LatestMilkRow> = sqlx::query_as(
        "SELECT a.id as animal_id, a.name, \
         TO_CHAR(m.date, 'YYYY-MM-DD') as date, \
         m.milk_amount, m.avg_amount, m.isk \
         FROM milk_day_productions m \
         JOIN animals a ON a.id = m.animal_id \
         WHERE m.date = (SELECT MAX(date) FROM milk_day_productions) \
         ORDER BY m.milk_amount DESC NULLS LAST \
         LIMIT 20",
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    Ok(rows
        .into_iter()
        .map(|r| LatestMilkEntry {
            animal_id: r.animal_id,
            name: r.name,
            date: r.date,
            milk_amount: r.milk_amount,
            avg_amount: r.avg_amount,
            isk: r.isk,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn seed_cow(pool: &PgPool) -> i32 {
        let row: (i32,) = sqlx::query_as(
            "INSERT INTO animals (gender, birth_date, active) VALUES ('female', '2020-01-01'::date, true) RETURNING id",
        )
        .fetch_one(pool)
        .await
        .unwrap();
        row.0
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_kpi_empty(pool: PgPool) {
        let res = kpi(&pool).await.unwrap();
        assert!(res.avg_calving_interval_days.is_none());
        assert!(res.conception_rate_pct.is_none());
        assert!(res.avg_milk_by_lactation.is_empty());
        assert!(res.feed_efficiency.is_none());
        assert!(res.avg_days_to_first_ai.is_none());
        assert!(res.avg_scc.is_none());
        assert!(res.refusal_rate_pct.is_none());
        assert!(res.culling_risk.is_empty());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_kpi_with_data(pool: PgPool) {
        let cow = seed_cow(&pool).await;
        let today = chrono::Utc::now().date_naive();

        sqlx::query(
            "INSERT INTO calvings (animal_id, calving_date, lac_number) VALUES ($1, $2, 1)",
        )
        .bind(cow)
        .bind(today - chrono::Duration::days(200))
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO calvings (animal_id, calving_date, lac_number) VALUES ($1, $2, 2)",
        )
        .bind(cow)
        .bind(today - chrono::Duration::days(100))
        .execute(&pool)
        .await
        .unwrap();

        for i in 0..10i64 {
            sqlx::query(
                "INSERT INTO milk_day_productions (animal_id, date, milk_amount) VALUES ($1, $2, 25.0)",
            )
            .bind(cow)
            .bind(today - chrono::Duration::days(i))
            .execute(&pool)
            .await
            .unwrap();
        }

        sqlx::query("INSERT INTO inseminations (animal_id, insemination_date) VALUES ($1, $2)")
            .bind(cow)
            .bind(today - chrono::Duration::days(150))
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query("INSERT INTO pregnancies (animal_id, pregnancy_date) VALUES ($1, $2)")
            .bind(cow)
            .bind(today - chrono::Duration::days(140))
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query(
            "INSERT INTO milk_quality (animal_id, date, scc, milkings, refusals) VALUES ($1, $2, 150000, 3, 0)",
        )
        .bind(cow)
        .bind(today - chrono::Duration::days(5))
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO feed_day_amounts (animal_id, feed_date, feed_number, total) VALUES ($1, $2, 1, 12.5)",
        )
        .bind(cow)
        .bind(today - chrono::Duration::days(5))
        .execute(&pool)
        .await
        .unwrap();

        let res = kpi(&pool).await.unwrap();
        assert_eq!(res.avg_calving_interval_days, Some(100.0));
        assert_eq!(res.conception_rate_pct, Some(100.0));
        assert_eq!(res.avg_milk_by_lactation.len(), 2);
        assert_eq!(res.feed_efficiency, Some(20.0));
        assert_eq!(res.avg_days_to_first_ai, Some(50.0));
        assert_eq!(res.avg_scc, Some(150000.0));
        assert_eq!(res.refusal_rate_pct, Some(0.0));
        assert!(res.culling_risk.is_empty());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_alerts_empty(pool: PgPool) {
        let res = alerts(&pool).await.unwrap();
        assert!(res.alerts.is_empty());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_milk_trend_empty(pool: PgPool) {
        let res = milk_trend(&pool, 30, 7).await.unwrap();
        assert!(res.daily.is_empty());
        assert!(res.forecast.is_empty());
        assert_eq!(res.trend_direction, "insufficient_data");
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_reproduction_forecast_empty(pool: PgPool) {
        let res = reproduction_forecast(&pool).await.unwrap();
        assert!(res.expected_calvings.is_empty());
        assert!(res.expected_heats.is_empty());
        assert!(res.dry_off_recommendations.is_empty());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_feed_forecast_empty(pool: PgPool) {
        let res = feed_forecast(&pool).await.unwrap();
        assert!(res.weekly_feed_kg.is_none());
        assert_eq!(res.predicted_next_week_kg, 0.0);
        assert!(res.avg_per_cow_day_kg.is_none());
        assert!(res.milk_per_feed.is_none());
    }
}
