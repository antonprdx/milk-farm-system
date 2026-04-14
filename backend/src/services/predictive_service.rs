use chrono::Datelike;
use sqlx::PgPool;

use crate::errors::AppError;
use crate::models::analytics::*;

pub async fn lactation_curves(
    pool: &PgPool,
    animal_id: Option<i32>,
) -> Result<Vec<LactationCurveResponse>, AppError> {
    let filter = if let Some(id) = animal_id {
        format!("AND a.id = {}", id)
    } else {
        String::new()
    };

    let rows: Vec<(i32, Option<String>, Option<String>, i32, String)> = sqlx::query_as(&format!(
        "SELECT a.id, a.name, a.life_number, c.lac_number, c.calving_date::text
         FROM calvings c
         JOIN animals a ON a.id = c.animal_id
         WHERE a.active = true AND a.gender = 'female'
         AND c.calving_date >= CURRENT_DATE - INTERVAL '400 days'
         AND NOT EXISTS (
             SELECT 1 FROM calvings c2 WHERE c2.animal_id = c.animal_id
             AND c2.calving_date > c.calving_date
         )
         {filter}
         ORDER BY a.name"
    ))
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    let today = chrono::Utc::now().date_naive();
    let mut results = Vec::new();

    for (aid, name, ln, lac, calving_str) in rows {
        let calving = match chrono::NaiveDate::parse_from_str(&calving_str, "%Y-%m-%d") {
            Ok(d) => d,
            Err(_) => continue,
        };
        let current_dim = (today - calving).num_days() as i32;
        if current_dim < 5 {
            continue;
        }

        let milk_rows: Vec<(i32, Option<f64>)> = sqlx::query_as(
            "SELECT (m.date - $1::date)::int as dim, m.milk_amount
             FROM milk_day_productions m
             WHERE m.animal_id = $2 AND m.date >= $1 AND m.date < $1 + INTERVAL '400 days'
             ORDER BY dim",
        )
        .bind(calving)
        .bind(aid)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)?;

        if milk_rows.len() < 5 {
            continue;
        }

        let actual_points: Vec<LactationPoint> = milk_rows
            .iter()
            .map(|(dim, milk)| LactationPoint {
                dim: *dim,
                milk: *milk,
            })
            .collect();

        let actual_map: std::collections::HashMap<i32, f64> = milk_rows
            .iter()
            .filter_map(|(dim, milk)| milk.map(|m| (*dim, m)))
            .collect();

        let (a, b, c) = fit_wood_model(&actual_map);

        let max_dim = (current_dim + 120).min(400);
        let mut fitted_curve = Vec::new();
        let mut forecast = Vec::new();
        let mut peak_milk = 0.0_f64;
        let mut peak_dim = 0_i32;

        for dim in 1..=max_dim {
            let val = wood_predict(a, b, c, dim);
            if val > peak_milk {
                peak_milk = val;
                peak_dim = dim;
            }
            let pt = LactationPoint {
                dim,
                milk: Some((val * 100.0).round() / 100.0),
            };
            if dim <= current_dim {
                fitted_curve.push(pt);
            } else {
                forecast.push(pt);
            }
        }

        let predicted_305 = (1..=305)
            .map(|d| wood_predict(a, b, c, d))
            .sum::<f64>();

        results.push(LactationCurveResponse {
            animal_id: aid,
            animal_name: name,
            life_number: ln,
            lac_number: lac,
            calving_date: calving_str,
            current_dim,
            peak_milk: Some((peak_milk * 100.0).round() / 100.0),
            peak_dim: Some(peak_dim),
            predicted_total_305d: Some((predicted_305 * 100.0).round() / 100.0),
            actual_points,
            fitted_curve,
            forecast,
        });

        if results.len() >= 50 {
            break;
        }
    }

    Ok(results)
}

fn fit_wood_model(data: &std::collections::HashMap<i32, f64>) -> (f64, f64, f64) {
    let log_data: Vec<(f64, f64)> = data
        .iter()
        .filter(|(_, v)| **v > 0.0)
        .map(|(&d, &m)| (d as f64, m.ln()))
        .collect();

    if log_data.len() < 3 {
        return (data.values().next().copied().unwrap_or(20.0), 0.2, 0.003);
    }

    let mut xtx = [[0.0_f64; 3]; 3];
    let mut xty = [0.0_f64; 3];

    for (t, y) in &log_data {
        let row = [1.0_f64, t.ln(), *t];
        for i in 0..3 {
            xty[i] += row[i] * y;
            for j in 0..3 {
                xtx[i][j] += row[i] * row[j];
            }
        }
    }

    let Some(beta) = solve_3x3(xtx, xty) else {
        return (data.values().next().copied().unwrap_or(20.0), 0.2, 0.003);
    };

    let ln_a = beta[0];
    let b_val = beta[1];
    let c_val = -beta[2];

    let a = ln_a.exp();
    (a, b_val, c_val.max(0.001).min(0.1))
}

fn solve_3x3(a: [[f64; 3]; 3], mut b: [f64; 3]) -> Option<[f64; 3]> {
    let n = 3;
    let mut m = a;
    for col in 0..n {
        let mut max_row = col;
        let mut max_val = m[col][col].abs();
        for row in (col + 1)..n {
            let v = m[row][col].abs();
            if v > max_val {
                max_val = v;
                max_row = row;
            }
        }
        if max_val < 1e-12 {
            return None;
        }
        if max_row != col {
            m.swap(col, max_row);
            b.swap(col, max_row);
        }
        let pivot = m[col][col];
        for row in (col + 1)..n {
            let factor = m[row][col] / pivot;
            for j in col..n {
                m[row][j] -= factor * m[col][j];
            }
            b[row] -= factor * b[col];
        }
    }
    let mut x = [0.0; 3];
    for i in (0..n).rev() {
        let mut s = b[i];
        for j in (i + 1)..n {
            s -= m[i][j] * x[j];
        }
        if m[i][i].abs() < 1e-12 {
            return None;
        }
        x[i] = s / m[i][i];
    }
    Some(x)
}

fn wood_predict(a: f64, b: f64, c: f64, dim: i32) -> f64 {
    let t = dim as f64;
    if t <= 0.0 {
        return 0.0;
    }
    a * t.powf(b) * (-c * t).exp()
}

pub async fn health_index(pool: &PgPool) -> Result<HealthIndexResponse, AppError> {
    let rows: Vec<(i32, Option<String>, Option<String>, Option<f64>, Option<f64>, Option<f64>, Option<f64>, Option<f64>, Option<f64>, Option<f64>, Option<f64>, Option<f64>)> = sqlx::query_as(
        "SELECT a.id, a.name, a.life_number,
                short_m.milk, long_m.milk, long_m.std as milk_std,
                short_r.rum, long_r.rum, long_r.std as rum_std,
                short_a.act, long_a.act, long_a.std as act_std
         FROM animals a
         LEFT JOIN LATERAL (
             SELECT AVG(m.milk_amount)::float8 as milk
             FROM milk_day_productions m WHERE m.animal_id = a.id AND m.date >= CURRENT_DATE - INTERVAL '3 days'
         ) short_m ON true
         LEFT JOIN LATERAL (
             SELECT AVG(m.milk_amount)::float8 as milk, STDDEV(m.milk_amount)::float8 as std
             FROM milk_day_productions m WHERE m.animal_id = a.id AND m.date >= CURRENT_DATE - INTERVAL '30 days'
         ) long_m ON true
         LEFT JOIN LATERAL (
             SELECT AVG(r.rumination_minutes)::float8 as rum, STDDEV(r.rumination_minutes)::float8 as std
             FROM ruminations r WHERE r.animal_id = a.id AND r.date >= CURRENT_DATE - INTERVAL '30 days'
             AND r.date < CURRENT_DATE - INTERVAL '3 days'
         ) short_r ON true
         LEFT JOIN LATERAL (
             SELECT AVG(r.rumination_minutes)::float8 as rum, STDDEV(r.rumination_minutes)::float8 as std
             FROM ruminations r WHERE r.animal_id = a.id AND r.date >= CURRENT_DATE - INTERVAL '30 days'
         ) long_r ON true
         LEFT JOIN LATERAL (
             SELECT AVG(act.activity_counter)::float8 as act
             FROM activities act WHERE act.animal_id = a.id AND act.activity_datetime >= CURRENT_DATE - INTERVAL '3 days'
         ) short_a ON true
         LEFT JOIN LATERAL (
             SELECT AVG(act.activity_counter)::float8 as act, STDDEV(act.activity_counter)::float8 as std
             FROM activities act WHERE act.animal_id = a.id AND act.activity_datetime >= CURRENT_DATE - INTERVAL '30 days'
             AND act.activity_datetime < CURRENT_DATE - INTERVAL '3 days'
         ) long_a ON true
         WHERE a.active = true AND a.gender = 'female'"
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    let scc_rows: Vec<(i32, f64, Option<f64>)> = sqlx::query_as(
        "SELECT a.id, recent.scc, baseline.avg_scc
         FROM animals a
         JOIN LATERAL (
             SELECT COALESCE(AVG(q.scc),0)::float8 as scc FROM milk_quality q
             WHERE q.animal_id = a.id AND q.date >= CURRENT_DATE - INTERVAL '3 days'
         ) recent ON true
         JOIN LATERAL (
             SELECT AVG(q.scc)::float8 as avg_scc FROM milk_quality q
             WHERE q.animal_id = a.id AND q.date >= CURRENT_DATE - INTERVAL '90 days'
             AND q.date < CURRENT_DATE - INTERVAL '3 days'
         ) baseline ON true
         WHERE a.active = true AND a.gender = 'female'"
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    let scc_map: std::collections::HashMap<i32, (f64, f64)> = scc_rows
        .into_iter()
        .filter(|(_, scc, avg)| *scc > 0.0 && avg.is_some())
        .map(|(id, scc, avg)| (id, (scc, avg.unwrap())))
        .collect();

    let mut cows = Vec::new();
    for (id, name, ln, short_milk, long_milk, milk_std, short_rum, long_rum, rum_std, short_act, long_act, act_std) in rows {
        let milk_z = zscore(short_milk, long_milk, milk_std);
        let rum_z = zscore(short_rum, long_rum, rum_std);
        let act_z = zscore(short_act, long_act, act_std);

        let (scc_z, scc_concern) = if let Some((recent, baseline)) = scc_map.get(&id) {
            let ratio = recent / baseline;
            let z = if *baseline > 0.0 { (*recent - *baseline) / (*baseline * 0.5) } else { 0.0 };
            (Some(z), ratio > 1.5)
        } else {
            (None, false)
        };

        let mut score = 100.0_f64;
        let mut concerns = Vec::new();

        if let Some(z) = milk_z {
            if z < -2.0 {
                score -= 30.0;
                concerns.push(("milk_drop".to_string(), z));
            } else if z < -1.0 {
                score -= 15.0;
                concerns.push(("milk_drop".to_string(), z));
            }
        }

        if let Some(z) = rum_z {
            if z < -2.0 {
                score -= 25.0;
                concerns.push(("rumination_drop".to_string(), z));
            } else if z < -1.0 {
                score -= 10.0;
                concerns.push(("rumination_drop".to_string(), z));
            }
        }

        if let Some(z) = act_z {
            if z < -2.0 {
                score -= 20.0;
                concerns.push(("activity_drop".to_string(), z));
            } else if z < -1.0 {
                score -= 10.0;
                concerns.push(("activity_drop".to_string(), z));
            }
        }

        if let Some(z) = scc_z {
            if z > 2.0 {
                score -= 25.0;
                concerns.push(("high_scc".to_string(), z));
            } else if z > 1.0 || scc_concern {
                score -= 10.0;
                concerns.push(("high_scc".to_string(), z));
            }
        }

        score = score.max(0.0).min(100.0);
        let risk_level = if score < 40.0 {
            "critical"
        } else if score < 60.0 {
            "high"
        } else if score < 80.0 {
            "moderate"
        } else {
            "low"
        };

        concerns.sort_by(|a, b| a.1.abs().partial_cmp(&b.1.abs()).unwrap_or(std::cmp::Ordering::Equal).reverse());

        cows.push(CowHealthIndex {
            animal_id: id,
            animal_name: name,
            life_number: ln,
            health_score: (score * 10.0).round() / 10.0,
            milk_deviation_zscore: milk_z.map(|z| (z * 100.0).round() / 100.0),
            rumination_deviation_zscore: rum_z.map(|z| (z * 100.0).round() / 100.0),
            activity_deviation_zscore: act_z.map(|z| (z * 100.0).round() / 100.0),
            scc_deviation_zscore: scc_z.map(|z| (z * 100.0).round() / 100.0),
            risk_level: risk_level.to_string(),
            top_concern: concerns.first().map(|c| c.0.clone()),
        });
    }

    cows.sort_by(|a, b| a.health_score.partial_cmp(&b.health_score).unwrap_or(std::cmp::Ordering::Equal));
    Ok(HealthIndexResponse { cows })
}

fn zscore(short: Option<f64>, long: Option<f64>, std: Option<f64>) -> Option<f64> {
    let s = short?;
    let l = long?;
    let sd = std?;
    if sd < 0.01 {
        return None;
    }
    Some((s - l) / sd)
}

pub async fn fertility_window(pool: &PgPool) -> Result<FertilityWindowResponse, AppError> {
    let rows: Vec<(i32, Option<String>, Option<String>, Option<i64>, Option<f64>, Option<f64>, Option<f64>)> = sqlx::query_as(
        "SELECT a.id, a.name, a.life_number,
                (CURRENT_DATE - latest_c.calving_date)::int8 as days_since_calving,
                act_ratio.ratio as activity_signal,
                rum_ratio.ratio as rumination_signal,
                milk_ratio.ratio as milk_signal
         FROM animals a
         LEFT JOIN LATERAL (
             SELECT calving_date FROM calvings c WHERE c.animal_id = a.id ORDER BY c.calving_date DESC LIMIT 1
         ) latest_c ON true
         LEFT JOIN LATERAL (
             SELECT (recent.act / NULLIF(baseline.act, 0))::float8 as ratio FROM (
                 SELECT AVG(activity_counter)::float8 as act FROM activities
                 WHERE animal_id = a.id AND activity_datetime >= CURRENT_DATE - INTERVAL '2 days'
             ) recent, (
                 SELECT AVG(activity_counter)::float8 as act FROM activities
                 WHERE animal_id = a.id AND activity_datetime >= CURRENT_DATE - INTERVAL '14 days'
                 AND activity_datetime < CURRENT_DATE - INTERVAL '2 days'
             ) baseline
         ) act_ratio ON true
         LEFT JOIN LATERAL (
             SELECT (recent.rum / NULLIF(baseline.rum, 0))::float8 as ratio FROM (
                 SELECT AVG(rumination_minutes)::float8 as rum FROM ruminations
                 WHERE animal_id = a.id AND date >= CURRENT_DATE - INTERVAL '2 days'
             ) recent, (
                 SELECT AVG(rumination_minutes)::float8 as rum FROM ruminations
                 WHERE animal_id = a.id AND date >= CURRENT_DATE - INTERVAL '14 days'
                 AND date < CURRENT_DATE - INTERVAL '2 days'
             ) baseline
         ) rum_ratio ON true
         LEFT JOIN LATERAL (
             SELECT (recent.milk / NULLIF(baseline.milk, 0))::float8 as ratio FROM (
                 SELECT AVG(milk_amount)::float8 as milk FROM milk_day_productions
                 WHERE animal_id = a.id AND date >= CURRENT_DATE - INTERVAL '2 days'
             ) recent, (
                 SELECT AVG(milk_amount)::float8 as milk FROM milk_day_productions
                 WHERE animal_id = a.id AND date >= CURRENT_DATE - INTERVAL '14 days'
                 AND date < CURRENT_DATE - INTERVAL '2 days'
             ) baseline
         ) milk_ratio ON true
         WHERE a.active = true AND a.gender = 'female'
         AND NOT EXISTS (
             SELECT 1 FROM pregnancies p WHERE p.animal_id = a.id
             AND p.pregnancy_date >= CURRENT_DATE - INTERVAL '300 days'
             AND NOT EXISTS (SELECT 1 FROM calvings c WHERE c.animal_id = a.id AND c.calving_date > p.pregnancy_date)
         )"
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    let mut cows = Vec::new();
    for (id, name, ln, dim, act_signal, rum_signal, milk_signal) in rows {
        let in_window = dim.map_or(false, |d| d >= 30 && d <= 150);

        let mut score = 0.0_f64;
        let act_high = act_signal.map_or(false, |s| s > 1.3);
        let rum_low = rum_signal.map_or(false, |s| s < 0.85);
        let milk_drop = milk_signal.map_or(false, |s| s < 0.9);

        if act_high {
            score += 40.0;
        }
        if rum_low {
            score += 30.0;
        }
        if milk_drop {
            score += 20.0;
        }
        if in_window {
            score += 10.0;
        }

        let status = if score >= 60.0 {
            "optimal"
        } else if score >= 30.0 {
            "approaching"
        } else if in_window {
            "in_window"
        } else {
            "outside_window"
        };

        cows.push(CowFertilityWindow {
            animal_id: id,
            animal_name: name,
            life_number: ln,
            days_since_calving: dim,
            activity_signal: act_signal.map(|v| (v * 100.0).round() / 100.0),
            rumination_signal: rum_signal.map(|v| (v * 100.0).round() / 100.0),
            milk_signal: milk_signal.map(|v| (v * 100.0).round() / 100.0),
            combined_score: (score * 10.0).round() / 10.0,
            window_status: status.to_string(),
        });
    }

    cows.sort_by(|a, b| b.combined_score.partial_cmp(&a.combined_score).unwrap_or(std::cmp::Ordering::Equal));
    Ok(FertilityWindowResponse { cows })
}

pub async fn profitability(
    pool: &PgPool,
    milk_price_per_liter: f64,
    feed_cost_per_kg: f64,
) -> Result<ProfitabilityResponse, AppError> {
    let rows: Vec<(i32, Option<String>, Option<String>, Option<f64>, Option<f64>)> = sqlx::query_as(
        "SELECT a.id, a.name, a.life_number,
                milk_30d.avg_milk,
                feed_30d.avg_feed
         FROM animals a
         LEFT JOIN LATERAL (
             SELECT AVG(m.milk_amount)::float8 as avg_milk
             FROM milk_day_productions m WHERE m.animal_id = a.id AND m.date >= CURRENT_DATE - INTERVAL '30 days'
         ) milk_30d ON true
         LEFT JOIN LATERAL (
             SELECT SUM(f.total)::float8 / NULLIF(COUNT(DISTINCT f.feed_date)::float8, 0) as avg_feed
             FROM feed_day_amounts f WHERE f.animal_id = a.id AND f.feed_date >= CURRENT_DATE - INTERVAL '30 days'
         ) feed_30d ON true
         WHERE a.active = true AND a.gender = 'female'
         ORDER BY a.name"
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    let mut cows = Vec::new();
    let mut total_margin = 0.0_f64;
    let mut margin_count = 0usize;

    for (id, name, ln, avg_milk, avg_feed) in rows {
        let milk_rev = avg_milk.map(|m| m * milk_price_per_liter);
        let feed_cost = avg_feed.map(|f| f * feed_cost_per_kg);

        let margin_day = match (milk_rev, feed_cost) {
            (Some(rev), Some(cost)) => Some(rev - cost),
            (Some(rev), None) => Some(rev),
            _ => None,
        };

        let margin_30d = margin_day.map(|m| m * 30.0);
        let feed_ratio = match (milk_rev, feed_cost) {
            (Some(rev), Some(cost)) if rev > 0.0 => Some(cost / rev),
            _ => None,
        };

        if let Some(m) = margin_day {
            total_margin += m;
            margin_count += 1;
        }

        cows.push(CowProfitability {
            animal_id: id,
            animal_name: name,
            life_number: ln,
            avg_daily_milk: avg_milk.map(|v| (v * 100.0).round() / 100.0),
            avg_daily_feed: avg_feed.map(|v| (v * 100.0).round() / 100.0),
            estimated_milk_revenue_day: milk_rev.map(|v| (v * 100.0).round() / 100.0),
            estimated_feed_cost_day: feed_cost.map(|v| (v * 100.0).round() / 100.0),
            estimated_margin_day: margin_day.map(|v| (v * 100.0).round() / 100.0),
            margin_30d: margin_30d.map(|v| (v * 100.0).round() / 100.0),
            feed_cost_ratio: feed_ratio.map(|v| (v * 100.0).round() / 100.0),
        });
    }

    let herd_avg = if margin_count > 0 {
        Some((total_margin / margin_count as f64 * 100.0).round() / 100.0)
    } else {
        None
    };

    cows.sort_by(|a, b| {
        b.estimated_margin_day
            .unwrap_or(f64::NEG_INFINITY)
            .partial_cmp(&a.estimated_margin_day.unwrap_or(f64::NEG_INFINITY))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(ProfitabilityResponse {
        cows,
        herd_avg_margin_day: herd_avg,
    })
}

pub async fn seasonal_decomposition(pool: &PgPool) -> Result<SeasonalResponse, AppError> {
    let rows: Vec<(i32, Option<f64>)> = sqlx::query_as(
        "SELECT EXTRACT(MONTH FROM date)::int as month, AVG(daily_total)::float8 as avg_milk
         FROM (
             SELECT date, SUM(milk_amount)::float8 as daily_total
             FROM milk_day_productions
             WHERE date >= CURRENT_DATE - INTERVAL '365 days'
             GROUP BY date
         ) sub
         GROUP BY EXTRACT(MONTH FROM date)
         ORDER BY month"
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    let overall_avg: Option<f64> = sqlx::query_scalar(
        "SELECT AVG(daily_total)::float8 FROM (
             SELECT date, SUM(milk_amount)::float8 as daily_total
             FROM milk_day_productions WHERE date >= CURRENT_DATE - INTERVAL '365 days' GROUP BY date
         ) sub"
    )
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?;

    let overall = overall_avg;
    let month_names = [
        "", "Январь", "Февраль", "Март", "Апрель", "Май", "Июнь",
        "Июль", "Август", "Сентябрь", "Октябрь", "Ноябрь", "Декабрь",
    ];

    let monthly_indices: Vec<MonthlyIndex> = (1..=12)
        .map(|m| {
            let found = rows.iter().find(|(month, _)| *month == m);
            MonthlyIndex {
                month: m,
                month_name: month_names[m as usize].to_string(),
                avg_daily_milk: found.and_then(|(_, avg)| *avg),
                seasonal_index: match (found.and_then(|(_, avg)| *avg), overall) {
                    (Some(avg), Some(oa)) if oa > 0.0 => Some((avg / oa * 100.0).round() / 100.0 as f64),
                    _ => None,
                },
            }
        })
        .collect();

    let current_month = chrono::Utc::now().date_naive().month() as usize;
    let current_factor = monthly_indices
        .get(current_month - 1)
        .and_then(|m| m.seasonal_index);

    let trend_7d: Option<f64> = sqlx::query_scalar(
        "SELECT AVG(daily_total)::float8 FROM (
             SELECT date, SUM(milk_amount)::float8 as daily_total
             FROM milk_day_productions WHERE date >= CURRENT_DATE - INTERVAL '7 days' GROUP BY date
         ) sub"
    )
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?;

    let trend_30d: Option<f64> = sqlx::query_scalar(
        "SELECT AVG(daily_total)::float8 FROM (
             SELECT date, SUM(milk_amount)::float8 as daily_total
             FROM milk_day_productions WHERE date >= CURRENT_DATE - INTERVAL '30 days' GROUP BY date
         ) sub"
    )
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?;

    Ok(SeasonalResponse {
        monthly_indices,
        trend_7d: trend_7d.map(|v: f64| (v * 100.0).round() / 100.0),
        trend_30d: trend_30d.map(|v: f64| (v * 100.0).round() / 100.0),
        current_seasonal_factor: current_factor,
    })
}

pub async fn mastitis_risk(pool: &PgPool) -> Result<MastitisRiskResponse, AppError> {
    let rows: Vec<(i32, Option<String>, Option<String>, Option<f64>, Option<f64>, Option<f64>, Option<f64>, Option<i64>)> = sqlx::query_as(
        "SELECT a.id, a.name, a.life_number,
                scc_latest.scc as recent_scc,
                scc_trend.ratio as scc_trend,
                cond.avg_cond as avg_conductivity,
                milk_dev.dev as milk_deviation,
                dim.days as dim_days
         FROM animals a
         LEFT JOIN LATERAL (
             SELECT AVG(q.scc)::float8 as scc FROM milk_quality q
             WHERE q.animal_id = a.id AND q.date >= CURRENT_DATE - INTERVAL '7 days'
         ) scc_latest ON true
         LEFT JOIN LATERAL (
             SELECT (recent.scc / NULLIF(baseline.scc, 0))::float8 as ratio FROM (
                 SELECT AVG(q.scc)::float8 as scc FROM milk_quality q
                 WHERE q.animal_id = a.id AND q.date >= CURRENT_DATE - INTERVAL '7 days'
             ) recent, (
                 SELECT AVG(q.scc)::float8 as scc FROM milk_quality q
                 WHERE q.animal_id = a.id AND q.date >= CURRENT_DATE - INTERVAL '90 days'
                 AND q.date < CURRENT_DATE - INTERVAL '7 days'
             ) baseline
         ) scc_trend ON true
         LEFT JOIN LATERAL (
             SELECT AVG(v.lf_conductivity + v.lr_conductivity + v.rf_conductivity + v.rr_conductivity)::float8 / 4.0 as avg_cond
             FROM milk_visit_quality v WHERE v.animal_id = a.id
             AND v.visit_datetime >= CURRENT_DATE - INTERVAL '7 days'
         ) cond ON true
         LEFT JOIN LATERAL (
             SELECT (recent.milk / NULLIF(baseline.milk, 0) - 1)::float8 as dev FROM (
                 SELECT AVG(m.milk_amount)::float8 as milk FROM milk_day_productions m
                 WHERE m.animal_id = a.id AND m.date >= CURRENT_DATE - INTERVAL '7 days'
             ) recent, (
                 SELECT AVG(m.milk_amount)::float8 as milk FROM milk_day_productions m
                 WHERE m.animal_id = a.id AND m.date >= CURRENT_DATE - INTERVAL '30 days'
                 AND m.date < CURRENT_DATE - INTERVAL '7 days'
             ) baseline
         ) milk_dev ON true
         LEFT JOIN LATERAL (
             SELECT (CURRENT_DATE - c.calving_date)::int8 as days
             FROM calvings c WHERE c.animal_id = a.id ORDER BY c.calving_date DESC LIMIT 1
         ) dim ON true
         WHERE a.active = true AND a.gender = 'female'"
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    let mut cows = Vec::new();
    for (id, name, ln, scc, scc_t, cond, milk_dev, dim) in rows {
        let Some(recent_scc) = scc else { continue };

        let mut score = 0.0_f64;
        let mut factors = Vec::new();

        if recent_scc > 500000.0 {
            score += 0.4;
            factors.push("SCC>500k".to_string());
        } else if recent_scc > 300000.0 {
            score += 0.25;
            factors.push("SCC>300k".to_string());
        } else if recent_scc > 200000.0 {
            score += 0.1;
            factors.push("SCC>200k".to_string());
        }

        if let Some(trend) = scc_t {
            if trend > 2.0 {
                score += 0.25;
                factors.push("SCC↑↑".to_string());
            } else if trend > 1.5 {
                score += 0.15;
                factors.push("SCC↑".to_string());
            }
        }

        if let Some(c) = cond {
            if c > 60.0 {
                score += 0.2;
                factors.push("conductivity↑".to_string());
            }
        }

        if let Some(dev) = milk_dev {
            if dev < -0.15 {
                score += 0.15;
                factors.push("milk↓".to_string());
            }
        }

        if let Some(d) = dim {
            if d < 30 {
                score += 0.1;
                factors.push("early_lactation".to_string());
            }
        }

        if score < 0.05 {
            continue;
        }

        score = score.min(1.0);
        let risk_level = if score >= 0.6 {
            "high"
        } else if score >= 0.3 {
            "medium"
        } else {
            "low"
        };

        cows.push(MastitisRiskEntry {
            animal_id: id,
            animal_name: name,
            life_number: ln,
            risk_score: (score * 100.0).round() / 100.0,
            risk_level: risk_level.to_string(),
            contributing_factors: factors,
        });
    }

    cows.sort_by(|a, b| b.risk_score.partial_cmp(&a.risk_score).unwrap_or(std::cmp::Ordering::Equal));
    Ok(MastitisRiskResponse {
        cows,
        model_version: "rule-based-v1".to_string(),
    })
}

pub async fn culling_survival(pool: &PgPool) -> Result<CullingSurvivalResponse, AppError> {
    let rows: Vec<(i32, Option<String>, Option<String>, Option<i64>, Option<f64>, Option<f64>, Option<f64>, Option<i64>)> = sqlx::query_as(
        "SELECT a.id, a.name, a.life_number,
                EXTRACT(YEAR FROM AGE(CURRENT_DATE, a.birth_date))::int8 as age_years,
                latest_milk.milk as recent_milk,
                avg_scc.scc as avg_scc,
                ci.interval as calving_interval,
                lac_count.lacs as lactation_count
         FROM animals a
         LEFT JOIN LATERAL (
             SELECT AVG(m.milk_amount)::float8 as milk FROM milk_day_productions m
             WHERE m.animal_id = a.id AND m.date >= CURRENT_DATE - INTERVAL '30 days'
         ) latest_milk ON true
         LEFT JOIN LATERAL (
             SELECT AVG(q.scc)::float8 as scc FROM milk_quality q
             WHERE q.animal_id = a.id AND q.date >= CURRENT_DATE - INTERVAL '90 days'
         ) avg_scc ON true
         LEFT JOIN LATERAL (
             SELECT AVG(c2.calving_date - c1.calving_date)::float8 as interval
             FROM calvings c1 JOIN calvings c2 ON c1.animal_id = c2.animal_id AND c2.calving_date > c1.calving_date
             WHERE c1.animal_id = a.id
             AND NOT EXISTS (SELECT 1 FROM calvings c3 WHERE c3.animal_id = c1.animal_id AND c3.calving_date > c1.calving_date AND c3.calving_date < c2.calving_date)
         ) ci ON true
         LEFT JOIN LATERAL (
             SELECT COUNT(*)::int8 as lacs FROM calvings c WHERE c.animal_id = a.id
         ) lac_count ON true
         WHERE a.active = true AND a.gender = 'female'"
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    let mut cows = Vec::new();
    for (id, name, ln, age, milk, scc, interval, lac_count) in rows {
        let mut risk = 0.0_f64;
        let mut factors = Vec::new();
        let mut base_days = 730.0_f64;

        if let Some(a) = age {
            if a >= 10 {
                risk += 0.4;
                base_days -= 365.0;
                factors.push("age>=10yr".to_string());
            } else if a >= 8 {
                risk += 0.25;
                base_days -= 200.0;
                factors.push("age>=8yr".to_string());
            } else if a >= 6 {
                risk += 0.1;
                base_days -= 100.0;
                factors.push("age>=6yr".to_string());
            }
        }

        if let Some(m) = milk {
            if m < 15.0 {
                risk += 0.3;
                base_days -= 180.0;
                factors.push("milk<15L".to_string());
            } else if m < 20.0 {
                risk += 0.1;
                base_days -= 60.0;
                factors.push("milk<20L".to_string());
            }
        }

        if let Some(s) = scc {
            if s > 300000.0 {
                risk += 0.25;
                base_days -= 180.0;
                factors.push("SCC>300k".to_string());
            } else if s > 200000.0 {
                risk += 0.1;
                base_days -= 90.0;
                factors.push("SCC>200k".to_string());
            }
        }

        if let Some(ci) = interval {
            if ci > 450.0 {
                risk += 0.2;
                base_days -= 120.0;
                factors.push("interval>450d".to_string());
            } else if ci > 400.0 {
                risk += 0.1;
                base_days -= 60.0;
                factors.push("interval>400d".to_string());
            }
        }

        if let Some(lc) = lac_count {
            if lc >= 6 {
                risk += 0.1;
                factors.push("lac>=6".to_string());
            }
        }

        if risk < 0.1 {
            continue;
        }

        risk = risk.min(1.0);
        let expected_days = (base_days * (1.0 - risk)).max(0.0) as i64;

        cows.push(CullingSurvivalEntry {
            animal_id: id,
            animal_name: name,
            life_number: ln,
            expected_days_remaining: Some(expected_days),
            risk_score: (risk * 100.0).round() / 100.0,
            risk_factors: factors,
        });
    }

    cows.sort_by(|a, b| b.risk_score.partial_cmp(&a.risk_score).unwrap_or(std::cmp::Ordering::Equal));
    Ok(CullingSurvivalResponse {
        cows,
        model_version: "rule-based-v1".to_string(),
    })
}
