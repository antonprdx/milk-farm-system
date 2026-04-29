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

fn r2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

fn interpolate_nans(data: &mut [f64]) {
    for i in 0..data.len() {
        if data[i].is_nan() {
            let left = (0..i).rev().find(|&j| !data[j].is_nan());
            let right = (i + 1..data.len()).find(|&j| !data[j].is_nan());
            match (left, right) {
                (Some(l), Some(r)) => {
                    let t = (i - l) as f64 / (r - l) as f64;
                    data[i] = data[l] * (1.0 - t) + data[r] * t;
                }
                (Some(l), None) => data[i] = data[l],
                (None, Some(r)) => data[i] = data[r],
                (None, None) => data[i] = 0.0,
            }
        }
    }
}

fn clean_outliers(values: &[f64]) -> Vec<f64> {
    if values.len() < 5 {
        return values.to_vec();
    }
    let mut cleaned: Vec<f64> = values.to_vec();
    for v in &mut cleaned {
        if *v <= 0.0 {
            *v = f64::NAN;
        }
    }
    interpolate_nans(&mut cleaned);

    let mut sorted: Vec<f64> = cleaned.iter().copied().filter(|v| !v.is_nan()).collect();
    if sorted.len() < 5 {
        return cleaned;
    }
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let q1 = sorted[sorted.len() / 4];
    let q3 = sorted[3 * sorted.len() / 4];
    let iqr = q3 - q1;
    if iqr < 1e-10 {
        return cleaned;
    }
    let lo = q1 - 1.5 * iqr;
    let hi = q3 + 1.5 * iqr;
    for v in &mut cleaned {
        if *v < lo || *v > hi {
            *v = f64::NAN;
        }
    }
    interpolate_nans(&mut cleaned);
    cleaned
}

fn holt_fit(values: &[f64], alpha: f64, beta: f64) -> (f64, f64, Vec<f64>) {
    let n = values.len();
    if n == 0 {
        return (0.0, 0.0, vec![]);
    }
    let mut level = values[0];
    let mut trend = if n > 1 { values[1] - values[0] } else { 0.0 };
    let mut fitted = vec![level];
    for &val in values.iter().skip(1) {
        fitted.push(level + trend);
        let new_level = alpha * val + (1.0 - alpha) * (level + trend);
        let new_trend = beta * (new_level - level) + (1.0 - beta) * trend;
        level = new_level;
        trend = new_trend;
    }
    (level, trend, fitted)
}

struct HWResult {
    level: f64,
    trend: f64,
    seasonal: Vec<f64>,
    fitted: Vec<f64>,
}

fn holt_winters_fit(
    values: &[f64],
    alpha: f64,
    beta: f64,
    gamma: f64,
    period: usize,
) -> HWResult {
    let n = values.len();
    let cycle_avg: f64 = values[0..period].iter().sum::<f64>() / period as f64;
    let mut level = cycle_avg;
    let mut trend = if n >= 2 * period {
        let avg2: f64 = values[period..2 * period].iter().sum::<f64>() / period as f64;
        (avg2 - cycle_avg) / period as f64
    } else {
        0.0
    };
    let mut seasonal: Vec<f64> = (0..period).map(|i| values[i] - cycle_avg).collect();
    let mut fitted = Vec::with_capacity(n);
    for (t, &val) in values.iter().enumerate() {
        let s_idx = t % period;
        let s_val = seasonal[s_idx];
        fitted.push(level + trend + s_val);
        let new_level = alpha * (val - s_val) + (1.0 - alpha) * (level + trend);
        let new_trend = beta * (new_level - level) + (1.0 - beta) * trend;
        let new_seasonal = gamma * (val - new_level) + (1.0 - gamma) * s_val;
        level = new_level;
        trend = new_trend;
        seasonal[s_idx] = new_seasonal;
    }
    HWResult {
        level,
        trend,
        seasonal,
        fitted,
    }
}

fn optimize_holt(values: &[f64]) -> (f64, f64) {
    let n = values.len();
    let train_len = ((n as f64) * 0.75) as usize;
    if train_len < 3 {
        return (0.3, 0.1);
    }
    let train = &values[..train_len];
    let val = &values[train_len..];
    let mut best_sse = f64::INFINITY;
    let mut best = (0.3, 0.1);
    for ai in 1..10u32 {
        let alpha = ai as f64 * 0.1;
        for bi in 1..6u32 {
            let beta = bi as f64 * 0.05;
            let (level, trend, _) = holt_fit(train, alpha, beta);
            let mut sse = 0.0;
            for (h, &v) in val.iter().enumerate() {
                let pred = level + (h + 1) as f64 * trend;
                let err = v - pred;
                sse += err * err;
            }
            if sse < best_sse {
                best_sse = sse;
                best = (alpha, beta);
            }
        }
    }
    best
}

fn optimize_hw(values: &[f64], period: usize) -> (f64, f64, f64) {
    let n = values.len();
    let train_len = ((n as f64) * 0.8) as usize;
    if train_len < 2 * period {
        return (0.3, 0.1, 0.1);
    }
    let train = &values[..train_len];
    let val = &values[train_len..];
    let mut best_sse = f64::INFINITY;
    let mut best = (0.3, 0.1, 0.1);
    for ai in 1..10u32 {
        let alpha = ai as f64 * 0.1;
        for bi in 1..6u32 {
            let beta = bi as f64 * 0.05;
            for gi in 1..6u32 {
                let gamma = gi as f64 * 0.05;
                let model = holt_winters_fit(train, alpha, beta, gamma, period);
                let mut sse = 0.0;
                for (h, &v) in val.iter().enumerate() {
                    let s_idx = (train_len + h) % period;
                    let pred =
                        model.level + (h + 1) as f64 * model.trend + model.seasonal[s_idx];
                    let err = v - pred;
                    sse += err * err;
                }
                if sse < best_sse {
                    best_sse = sse;
                    best = (alpha, beta, gamma);
                }
            }
        }
    }
    best
}

fn compute_rmse(residuals: &[f64]) -> f64 {
    let skip = 2.min(residuals.len().saturating_sub(1));
    if residuals.len() <= skip {
        return 0.0;
    }
    let sse: f64 = residuals[skip..].iter().map(|r| r * r).sum();
    (sse / (residuals.len() - skip) as f64).sqrt()
}

fn compute_mape(actual: &[f64], fitted: &[f64]) -> f64 {
    let skip = 2.min(actual.len().saturating_sub(1));
    if actual.len() <= skip {
        return 100.0;
    }
    let n = actual.len() - skip;
    let sum: f64 = actual[skip..]
        .iter()
        .zip(fitted[skip..].iter())
        .map(|(a, f)| {
            if a.abs() > 1e-10 {
                (a - f).abs() / a.abs()
            } else {
                0.0
            }
        })
        .sum();
    (sum / n as f64) * 100.0
}

fn detect_structural_breaks(values: &[f64], dates: &[String]) -> Vec<BreakPoint> {
    let n = values.len();
    if n < 14 {
        return vec![];
    }
    let window = 7.min(n / 3);
    let mut breaks = Vec::new();
    let mut last_idx = 0;
    for i in window..(n - window) {
        let before: f64 = values[(i - window)..i].iter().sum::<f64>() / window as f64;
        let after: f64 = values[i..(i + window)].iter().sum::<f64>() / window as f64;
        let b_var: f64 = values[(i - window)..i]
            .iter()
            .map(|v| (v - before).powi(2))
            .sum::<f64>()
            / window as f64;
        let a_var: f64 = values[i..(i + window)]
            .iter()
            .map(|v| (v - after).powi(2))
            .sum::<f64>()
            / window as f64;
        let pooled = ((b_var + a_var) / 2.0).sqrt();
        if pooled < 1e-10 {
            continue;
        }
        let diff = after - before;
        let t_stat = diff.abs() / (pooled * (2.0 / window as f64).sqrt());
        if t_stat > 3.0 && i > last_idx + window {
            let direction = if diff > 0.0 {
                "increase"
            } else {
                "decrease"
            };
            let mag = if before.abs() > 1e-10 {
                (diff / before * 100.0).abs()
            } else {
                0.0
            };
            breaks.push(BreakPoint {
                date: dates.get(i).cloned().unwrap_or_default(),
                index: i as i32,
                direction: direction.to_string(),
                magnitude: r2(mag),
            });
            last_idx = i;
        }
    }
    breaks
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

    let (dates, raw_values): (Vec<String>, Vec<f64>) = daily_pts
        .iter()
        .filter_map(|p| p.total_milk.map(|v| (p.date.clone(), v)))
        .unzip();

    if raw_values.len() < 7 {
        return Ok(MilkTrendResponse {
            daily: daily_pts,
            forecast: vec![],
            trend_direction: "insufficient_data".to_string(),
            trend_percent: 0.0,
            confidence: 0.0,
            mape: 100.0,
            model_type: "insufficient".to_string(),
            structural_breaks: vec![],
        });
    }

    let cleaned = clean_outliers(&raw_values);
    let period = 7;

    let (level, trend, seasonal, fitted, model_type) = if cleaned.len() >= 2 * period {
        let (alpha, beta, gamma) = optimize_hw(&cleaned, period);
        let model = holt_winters_fit(&cleaned, alpha, beta, gamma, period);
        (
            model.level,
            model.trend,
            model.seasonal,
            model.fitted,
            "holt_winters",
        )
    } else {
        let (alpha, beta) = optimize_holt(&cleaned);
        let (level, trend, fitted) = holt_fit(&cleaned, alpha, beta);
        (level, trend, vec![], fitted, "holt")
    };

    let residuals: Vec<f64> = cleaned
        .iter()
        .zip(fitted.iter())
        .map(|(a, f)| a - f)
        .collect();
    let rmse = compute_rmse(&residuals);
    let mape_val = compute_mape(&cleaned, &fitted);

    let last_date = daily_pts
        .last()
        .and_then(|p| chrono::NaiveDate::parse_from_str(&p.date, "%Y-%m-%d").ok());

    let z = 1.96;
    let mut fc = Vec::new();
    if let Some(ld) = last_date {
        for h in 1..=forecast_days {
            let t_idx = cleaned.len() + h as usize - 1;
            let s_comp = if !seasonal.is_empty() {
                seasonal[t_idx % period]
            } else {
                0.0
            };
            let pred = level + h as f64 * trend + s_comp;
            let err = z * rmse * (1.0 + h as f64 * 0.1).sqrt();
            fc.push(ForecastPoint {
                date: (ld + chrono::Duration::days(h))
                    .format("%Y-%m-%d")
                    .to_string(),
                predicted: r2(pred),
                lower: r2(pred - err),
                upper: r2(pred + err),
            });
        }
    }

    let trend_pct = if level.abs() > 1e-10 {
        (trend / level * 100.0).clamp(-100.0, 100.0)
    } else {
        0.0
    };
    let direction = if trend_pct > 5.0 {
        "significant_up"
    } else if trend_pct > 2.0 {
        "up"
    } else if trend_pct < -5.0 {
        "significant_down"
    } else if trend_pct < -2.0 {
        "down"
    } else {
        "stable"
    };

    let structural_breaks = detect_structural_breaks(&cleaned, &dates);

    Ok(MilkTrendResponse {
        daily: daily_pts,
        forecast: fc,
        trend_direction: direction.to_string(),
        trend_percent: r2(trend_pct),
        confidence: r2(rmse),
        mape: r2(mape_val),
        model_type: model_type.to_string(),
        structural_breaks,
    })
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

pub async fn time_series_comparison(
    pool: &PgPool,
    animal_id: i32,
    days: i64,
    forecast_days: i64,
) -> Result<TimeSeriesComparisonResponse, AppError> {
    let name: Option<String> = sqlx::query_scalar(
        "SELECT a.name FROM animals a WHERE a.id = $1",
    )
    .bind(animal_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?
    .flatten();

    let rows: Vec<(String, Option<f64>)> = sqlx::query_as(
        "SELECT date::text, milk_amount::float8
         FROM milk_day_productions
         WHERE animal_id = $1 AND date >= CURRENT_DATE - ($2 || ' days')::interval
         ORDER BY date",
    )
    .bind(animal_id)
    .bind(days)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    let dates: Vec<String> = rows.iter().map(|(d, _)| d.clone()).collect();
    let raw_values: Vec<f64> = rows.iter().filter_map(|(_, v)| *v).collect();

    if raw_values.len() < 10 {
        return Ok(TimeSeriesComparisonResponse {
            animal_id,
            animal_name: name,
            actual_dates: dates,
            actual_values: raw_values,
            models: vec![],
            best_model: "insufficient_data".to_string(),
        });
    }

    let values = clean_outliers(&raw_values);
    let dates_clean: Vec<String> = dates[..values.len()].to_vec();

    let period = 7;
    let train_len = ((values.len() as f64) * 0.8) as usize;
    if train_len < 10 {
        return Ok(TimeSeriesComparisonResponse {
            animal_id,
            animal_name: name,
            actual_dates: dates_clean,
            actual_values: values,
            models: vec![],
            best_model: "insufficient_data".to_string(),
        });
    }

    let train = &values[..train_len];
    let test = &values[train_len..];

    let last_date = chrono::NaiveDate::parse_from_str(
        dates_clean.last().unwrap_or(&String::new()),
        "%Y-%m-%d",
    );

    let mut models = Vec::new();

    models.push(fit_ses(train, test, &dates_clean, forecast_days, last_date.ok()));
    models.push(fit_sma(train, test, &dates_clean, forecast_days, last_date.ok(), 7));
    models.push(fit_wma(train, test, &dates_clean, forecast_days, last_date.ok(), 7));
    models.push(fit_linear_regression(train, test, &dates_clean, forecast_days, last_date.ok()));
    models.push(fit_double_exponential(train, test, &dates_clean, forecast_days, last_date.ok()));
    if values.len() >= 2 * period {
        models.push(fit_holt_winters_model(train, test, &dates_clean, forecast_days, last_date.ok(), period));
    }
    models.push(fit_fourier(train, test, &dates_clean, forecast_days, last_date.ok(), 3));
    models.push(fit_arima_011(train, test, &dates_clean, forecast_days, last_date.ok()));

    let best = models
        .iter()
        .min_by(|a, b| a.mape.partial_cmp(&b.mape).unwrap_or(std::cmp::Ordering::Equal))
        .map(|m| m.model_name.clone())
        .unwrap_or_else(|| "none".to_string());

    Ok(TimeSeriesComparisonResponse {
        animal_id,
        animal_name: name,
        actual_dates: dates_clean,
        actual_values: values,
        models,
        best_model: best,
    })
}

fn make_forecast_dates(last_date: Option<chrono::NaiveDate>, actual_len: usize, forecast_days: i64) -> Vec<String> {
    if let Some(ld) = last_date {
        (1..=forecast_days)
            .map(|h| (ld + chrono::Duration::days(h)).format("%Y-%m-%d").to_string())
            .collect()
    } else {
        (1..=forecast_days)
            .map(|h| format!("day_{}", actual_len + h as usize))
            .collect()
    }
}

fn calc_metrics(actual: &[f64], fitted: &[f64]) -> (f64, f64) {
    let n = actual.len().min(fitted.len());
    if n == 0 { return (100.0, 0.0); }
    let sse: f64 = (0..n).map(|i| (actual[i] - fitted[i]).powi(2)).sum();
    let rmse = (sse / n as f64).sqrt();
    let mape_sum: f64 = (0..n).map(|i| {
        if actual[i].abs() > 1e-10 { (actual[i] - fitted[i]).abs() / actual[i].abs() } else { 0.0 }
    }).sum();
    let mape = (mape_sum / n as f64) * 100.0;
    (r2(mape), r2(rmse))
}

fn fit_ses(
    train: &[f64], test: &[f64], dates: &[String], forecast_days: i64, last_date: Option<chrono::NaiveDate>,
) -> ModelResult {
    let mut best_alpha = 0.3_f64;
    let mut best_sse = f64::INFINITY;
    for ai in 1..20u32 {
        let alpha = ai as f64 * 0.05;
        let mut level = train[0];
        let mut sse = 0.0;
        for &v in train.iter().skip(1) {
            let pred = level;
            let err = v - pred;
            sse += err * err;
            level = alpha * v + (1.0 - alpha) * level;
        }
        for &v in test.iter() {
            let pred = level;
            let err = v - pred;
            sse += err * err;
        }
        if sse < best_sse {
            best_sse = sse;
            best_alpha = alpha;
        }
    }

    let mut level = train[0];
    let mut fitted_vals: Vec<f64> = vec![level];
    for &v in train.iter().skip(1) {
        fitted_vals.push(level);
        level = best_alpha * v + (1.0 - best_alpha) * level;
    }

    let test_fitted: Vec<f64> = test.iter().map(|_| level).collect();
    let (mape, rmse) = calc_metrics(test, &test_fitted);

    let implied_slope = if fitted_vals.len() >= 3 {
        let recent = &fitted_vals[fitted_vals.len() - 3..];
        (recent[2] - recent[0]) / 2.0
    } else {
        0.0
    };

    let fc_dates = make_forecast_dates(last_date, dates.len(), forecast_days);
    let forecast: Vec<ModelForecastPoint> = fc_dates.iter().enumerate().map(|(h, d)| ModelForecastPoint {
        date: d.clone(),
        value: r2(level + implied_slope * (h + 1) as f64),
    }).collect();

    let fitted_pts: Vec<ModelForecastPoint> = dates.iter().zip(fitted_vals.iter())
        .map(|(d, v)| ModelForecastPoint { date: d.clone(), value: r2(*v) })
        .collect();

    ModelResult {
        model_name: "SES".to_string(),
        description: format!("Simple Exponential Smoothing (α={:.2})", best_alpha),
        mape, rmse, forecast, fitted: fitted_pts,
    }
}

fn fit_sma(
    train: &[f64], test: &[f64], dates: &[String], forecast_days: i64, last_date: Option<chrono::NaiveDate>, window: usize,
) -> ModelResult {
    let all_train: Vec<f64> = train.to_vec();
    let mut fitted_vals: Vec<f64> = Vec::new();
    for i in 0..all_train.len() {
        if i < window {
            let avg: f64 = all_train[..=i].iter().sum::<f64>() / (i + 1) as f64;
            fitted_vals.push(avg);
        } else {
            let avg: f64 = all_train[(i - window + 1)..=i].iter().sum::<f64>() / window as f64;
            fitted_vals.push(avg);
        }
    }

    let last_avg: f64 = if all_train.len() >= window {
        all_train[all_train.len() - window..].iter().sum::<f64>() / window as f64
    } else {
        all_train.iter().sum::<f64>() / all_train.len() as f64
    };

    let test_fitted: Vec<f64> = test.iter().map(|_| last_avg).collect();
    let (mape, rmse) = calc_metrics(test, &test_fitted);

    let implied_slope = if fitted_vals.len() >= window * 2 {
        let recent_a = fitted_vals[fitted_vals.len() - window..].iter().sum::<f64>() / window as f64;
        let prev_a = fitted_vals[fitted_vals.len() - window * 2..fitted_vals.len() - window].iter().sum::<f64>() / window as f64;
        (recent_a - prev_a) / window as f64
    } else if fitted_vals.len() >= 2 {
        fitted_vals[fitted_vals.len() - 1] - fitted_vals[fitted_vals.len() - 2]
    } else {
        0.0
    };

    let fc_dates = make_forecast_dates(last_date, dates.len(), forecast_days);
    let forecast: Vec<ModelForecastPoint> = fc_dates.iter().enumerate().map(|(h, d)| ModelForecastPoint {
        date: d.clone(), value: r2(last_avg + implied_slope * (h + 1) as f64),
    }).collect();

    let fitted_pts: Vec<ModelForecastPoint> = dates.iter().zip(fitted_vals.iter())
        .map(|(d, v)| ModelForecastPoint { date: d.clone(), value: r2(*v) })
        .collect();

    ModelResult {
        model_name: "SMA".to_string(),
        description: format!("Simple Moving Average (window={})", window),
        mape, rmse, forecast, fitted: fitted_pts,
    }
}

fn fit_wma(
    train: &[f64], test: &[f64], dates: &[String], forecast_days: i64, last_date: Option<chrono::NaiveDate>, window: usize,
) -> ModelResult {
    let all_train = train.to_vec();
    let mut fitted_vals: Vec<f64> = Vec::new();
    let weights: Vec<f64> = (1..=window).map(|i| i as f64).collect();
    let w_sum: f64 = weights.iter().sum();

    for i in 0..all_train.len() {
        if i < window {
            fitted_vals.push(all_train[..=i].iter().sum::<f64>() / (i + 1) as f64);
        } else {
            let segment = &all_train[(i - window + 1)..=i];
            let wma: f64 = segment.iter().zip(weights.iter()).map(|(&v, &w)| v * w).sum::<f64>() / w_sum;
            fitted_vals.push(wma);
        }
    }

    let last_val = if all_train.len() >= window {
        let segment = &all_train[all_train.len() - window..];
        segment.iter().zip(weights.iter()).map(|(&v, &w)| v * w).sum::<f64>() / w_sum
    } else {
        all_train.iter().sum::<f64>() / all_train.len() as f64
    };

    let test_fitted: Vec<f64> = test.iter().map(|_| last_val).collect();
    let (mape, rmse) = calc_metrics(test, &test_fitted);

    let implied_slope = if fitted_vals.len() >= window * 2 {
        let recent_a = fitted_vals[fitted_vals.len() - window..].iter().sum::<f64>() / window as f64;
        let prev_a = fitted_vals[fitted_vals.len() - window * 2..fitted_vals.len() - window].iter().sum::<f64>() / window as f64;
        (recent_a - prev_a) / window as f64
    } else if fitted_vals.len() >= 2 {
        fitted_vals[fitted_vals.len() - 1] - fitted_vals[fitted_vals.len() - 2]
    } else {
        0.0
    };

    let fc_dates = make_forecast_dates(last_date, dates.len(), forecast_days);
    let forecast: Vec<ModelForecastPoint> = fc_dates.iter().enumerate().map(|(h, d)| ModelForecastPoint {
        date: d.clone(), value: r2(last_val + implied_slope * (h + 1) as f64),
    }).collect();

    let fitted_pts: Vec<ModelForecastPoint> = dates.iter().zip(fitted_vals.iter())
        .map(|(d, v)| ModelForecastPoint { date: d.clone(), value: r2(*v) })
        .collect();

    ModelResult {
        model_name: "WMA".to_string(),
        description: format!("Weighted Moving Average (window={})", window),
        mape, rmse, forecast, fitted: fitted_pts,
    }
}

fn fit_linear_regression(
    train: &[f64], test: &[f64], dates: &[String], forecast_days: i64, last_date: Option<chrono::NaiveDate>,
) -> ModelResult {
    let n = train.len() as f64;
    let sum_x: f64 = (0..train.len()).map(|i| i as f64).sum();
    let sum_y: f64 = train.iter().sum();
    let sum_xy: f64 = (0..train.len()).map(|i| i as f64 * train[i]).sum();
    let sum_x2: f64 = (0..train.len()).map(|i| (i as f64).powi(2)).sum();

    let denom = n * sum_x2 - sum_x * sum_x;
    let slope = slope_from_lr(n, sum_x, sum_y, sum_xy, sum_x2);
    let intercept = if denom.abs() > 1e-12 { (sum_y - slope * sum_x) / n } else { sum_y / n };

    let fitted_vals: Vec<f64> = (0..train.len()).map(|i| intercept + slope * i as f64).collect();
    let test_fitted: Vec<f64> = (train.len()..train.len() + test.len()).map(|i| intercept + slope * i as f64).collect();
    let (mape, rmse) = calc_metrics(test, &test_fitted);

    let fc_dates = make_forecast_dates(last_date, dates.len(), forecast_days);
    let start_idx = train.len() + test.len();
    let forecast: Vec<ModelForecastPoint> = fc_dates.iter().enumerate().map(|(h, d)| ModelForecastPoint {
        date: d.clone(), value: r2(intercept + slope * (start_idx + h) as f64),
    }).collect();

    let fitted_pts: Vec<ModelForecastPoint> = dates.iter().zip(fitted_vals.iter())
        .map(|(d, v)| ModelForecastPoint { date: d.clone(), value: r2(*v) })
        .collect();

    ModelResult {
        model_name: "LinearRegression".to_string(),
        description: format!("Linear Regression (slope={:.3})", slope),
        mape, rmse, forecast, fitted: fitted_pts,
    }
}

fn slope_from_lr(n: f64, sum_x: f64, sum_y: f64, sum_xy: f64, sum_x2: f64) -> f64 {
    let denom = n * sum_x2 - sum_x * sum_x;
    if denom.abs() > 1e-12 { (n * sum_xy - sum_x * sum_y) / denom } else { 0.0 }
}

fn fit_double_exponential(
    train: &[f64], test: &[f64], dates: &[String], forecast_days: i64, last_date: Option<chrono::NaiveDate>,
) -> ModelResult {
    let (alpha, beta) = optimize_holt(train);
    let (level, trend, fitted_raw) = holt_fit(train, alpha, beta);

    let test_fitted: Vec<f64> = test.iter().enumerate().map(|(h, _)| level + (h + 1) as f64 * trend).collect();
    let (mape, rmse) = calc_metrics(test, &test_fitted);

    let fc_dates = make_forecast_dates(last_date, dates.len(), forecast_days);
    let start_h = test.len() + 1;
    let forecast: Vec<ModelForecastPoint> = fc_dates.iter().enumerate().map(|(h, d)| ModelForecastPoint {
        date: d.clone(), value: r2(level + (start_h + h) as f64 * trend),
    }).collect();

    let fitted_pts: Vec<ModelForecastPoint> = dates.iter().zip(fitted_raw.iter())
        .map(|(d, v)| ModelForecastPoint { date: d.clone(), value: r2(*v) })
        .collect();

    ModelResult {
        model_name: "Holt".to_string(),
        description: format!("Holt's Linear (α={:.2}, β={:.2})", alpha, beta),
        mape, rmse, forecast, fitted: fitted_pts,
    }
}

fn fit_holt_winters_model(
    train: &[f64], test: &[f64], dates: &[String], forecast_days: i64, last_date: Option<chrono::NaiveDate>, period: usize,
) -> ModelResult {
    let (alpha, beta, gamma) = optimize_hw(train, period);
    let model = holt_winters_fit(train, alpha, beta, gamma, period);

    let train_len = train.len();
    let test_fitted: Vec<f64> = test.iter().enumerate().map(|(h, _)| {
        let s_idx = (train_len + h) % period;
        model.level + (h + 1) as f64 * model.trend + model.seasonal[s_idx]
    }).collect();
    let (mape, rmse) = calc_metrics(test, &test_fitted);

    let fc_dates = make_forecast_dates(last_date, dates.len(), forecast_days);
    let start_h = test.len() + 1;
    let forecast: Vec<ModelForecastPoint> = fc_dates.iter().enumerate().map(|(h, d)| {
        let s_idx = (train_len + start_h + h - 1) % period;
        ModelForecastPoint { date: d.clone(), value: r2(model.level + (start_h + h) as f64 * model.trend + model.seasonal[s_idx]) }
    }).collect();

    let fitted_pts: Vec<ModelForecastPoint> = dates.iter().zip(model.fitted.iter())
        .map(|(d, v)| ModelForecastPoint { date: d.clone(), value: r2(*v) })
        .collect();

    ModelResult {
        model_name: "HoltWinters".to_string(),
        description: format!("Holt-Winters (α={:.2}, β={:.2}, γ={:.2}, p={})", alpha, beta, gamma, period),
        mape, rmse, forecast, fitted: fitted_pts,
    }
}

fn fit_fourier(
    train: &[f64], test: &[f64], dates: &[String], forecast_days: i64, last_date: Option<chrono::NaiveDate>, harmonics: usize,
) -> ModelResult {
    let n = train.len() as f64;
    let _mean: f64 = train.iter().sum::<f64>() / n;

    let (slope, intercept) = {
        let sum_x: f64 = (0..train.len()).map(|i| i as f64).sum();
        let sum_y: f64 = train.iter().sum();
        let sum_xy: f64 = (0..train.len()).map(|i| i as f64 * train[i]).sum();
        let sum_x2: f64 = (0..train.len()).map(|i| (i as f64).powi(2)).sum();
        let s = slope_from_lr(n, sum_x, sum_y, sum_xy, sum_x2);
        let d = n * sum_x2 - sum_x * sum_x;
        let ic = if d.abs() > 1e-12 { (sum_y - s * sum_x) / n } else { sum_y / n };
        (s, ic)
    };

    let detrended: Vec<f64> = train.iter().enumerate().map(|(i, &v)| v - (intercept + slope * i as f64)).collect();

    let base_period = 7.0_f64;
    let mut cos_coeffs = vec![0.0_f64; harmonics];
    let mut sin_coeffs = vec![0.0_f64; harmonics];

    for k in 0..harmonics {
        let period = base_period / (k as f64 + 1.0).min(1.0);
        let mut cc = 0.0_f64;
        let mut sc = 0.0_f64;
        for (i, &v) in detrended.iter().enumerate() {
            let t = i as f64;
            cc += v * (2.0 * std::f64::consts::PI * t / period).cos();
            sc += v * (2.0 * std::f64::consts::PI * t / period).sin();
        }
        cos_coeffs[k] = 2.0 * cc / n;
        sin_coeffs[k] = 2.0 * sc / n;
    }

    let predict = |idx: usize| -> f64 {
        let mut val = intercept + slope * idx as f64;
        let t = idx as f64;
        for k in 0..harmonics {
            let period = base_period / (k as f64 + 1.0).min(1.0);
            let angle = 2.0 * std::f64::consts::PI * t / period;
            val += cos_coeffs[k] * angle.cos() + sin_coeffs[k] * angle.sin();
        }
        val
    };

    let fitted_vals: Vec<f64> = (0..train.len()).map(|i| predict(i)).collect();
    let test_fitted: Vec<f64> = (train.len()..train.len() + test.len()).map(|i| predict(i)).collect();
    let (mape, rmse) = calc_metrics(test, &test_fitted);

    let fc_dates = make_forecast_dates(last_date, dates.len(), forecast_days);
    let start_idx = train.len() + test.len();
    let forecast: Vec<ModelForecastPoint> = fc_dates.iter().enumerate().map(|(h, d)| ModelForecastPoint {
        date: d.clone(), value: r2(predict(start_idx + h)),
    }).collect();

    let fitted_pts: Vec<ModelForecastPoint> = dates.iter().zip(fitted_vals.iter())
        .map(|(d, v)| ModelForecastPoint { date: d.clone(), value: r2(*v) })
        .collect();

    ModelResult {
        model_name: "Fourier".to_string(),
        description: format!("Fourier Series ({} harmonics)", harmonics),
        mape, rmse, forecast, fitted: fitted_pts,
    }
}

fn fit_arima_011(
    train: &[f64], test: &[f64], dates: &[String], forecast_days: i64, last_date: Option<chrono::NaiveDate>,
) -> ModelResult {
    let mut diffed: Vec<f64> = Vec::with_capacity(train.len() - 1);
    for i in 1..train.len() {
        diffed.push(train[i] - train[i - 1]);
    }

    let mut best_theta = 0.5_f64;
    let mut best_sse = f64::INFINITY;
    for ti in 0..20u32 {
        let theta = ti as f64 * 0.05;
        let mut errors = vec![0.0_f64; diffed.len()];
        let mut sse = 0.0;
        for i in 0..diffed.len() {
            let pred = if i > 0 { theta * errors[i - 1] } else { 0.0 };
            errors[i] = diffed[i] - pred;
            sse += errors[i] * errors[i];
        }
        if sse < best_sse {
            best_sse = sse;
            best_theta = theta;
        }
    }

    let mut errors = vec![0.0_f64; diffed.len()];
    let mut fitted_vals = vec![train[0]];
    for i in 0..diffed.len() {
        let pred = if i > 0 { best_theta * errors[i - 1] } else { 0.0 };
        errors[i] = diffed[i] - pred;
        fitted_vals.push(train[i] + pred);
    }

    let mut last_error = *errors.last().unwrap_or(&0.0);
    let mut last_val = *train.last().unwrap_or(&0.0);
    let test_fitted: Vec<f64> = test.iter().map(|&v| {
        let pred_diff = best_theta * last_error;
        let pred = last_val + pred_diff;
        let actual_diff = v - last_val;
        last_error = actual_diff - pred_diff;
        last_val = v;
        pred
    }).collect();
    let (mape, rmse) = calc_metrics(test, &test_fitted);

    let fc_dates = make_forecast_dates(last_date, dates.len(), forecast_days);
    let mut f_val = *train.last().unwrap_or(&0.0);
    let mut f_err = *errors.last().unwrap_or(&0.0);
    let drift: f64 = diffed.iter().sum::<f64>() / diffed.len().max(1) as f64;
    let forecast: Vec<ModelForecastPoint> = fc_dates.iter().map(|d| {
        let pred = f_val + drift + best_theta * f_err;
        f_err = 0.0;
        f_val = pred;
        ModelForecastPoint { date: d.clone(), value: r2(pred) }
    }).collect();

    let fitted_pts: Vec<ModelForecastPoint> = dates.iter().zip(fitted_vals.iter())
        .map(|(d, v)| ModelForecastPoint { date: d.clone(), value: r2(*v) })
        .collect();

    ModelResult {
        model_name: "ARIMA(0,1,1)".to_string(),
        description: format!("ARIMA(0,1,1) (θ={:.2})", best_theta),
        mape, rmse, forecast, fitted: fitted_pts,
    }
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
        assert_eq!(res.model_type, "insufficient");
        assert!(res.structural_breaks.is_empty());
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
