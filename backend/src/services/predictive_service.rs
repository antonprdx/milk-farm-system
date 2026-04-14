use chrono::Datelike;
use sqlx::PgPool;

use crate::errors::AppError;
use crate::models::analytics::*;

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

pub async fn lactation_curves(
    pool: &PgPool,
    animal_id: Option<i32>,
) -> Result<Vec<LactationCurveResponse>, AppError> {
    let rows: Vec<(i32, Option<String>, Option<String>, i32, String)> = if let Some(id) = animal_id {
        sqlx::query_as(
            "SELECT a.id, a.name, a.life_number, c.lac_number, c.calving_date::text
             FROM calvings c
             JOIN animals a ON a.id = c.animal_id
             WHERE a.active = true AND a.gender = 'female' AND a.id = $1
             AND NOT EXISTS (
                 SELECT 1 FROM calvings c2 WHERE c2.animal_id = c.animal_id
                 AND c2.calving_date > c.calving_date
             )
             ORDER BY a.name",
        )
        .bind(id)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as(
            "SELECT a.id, a.name, a.life_number, c.lac_number, c.calving_date::text
             FROM calvings c
             JOIN animals a ON a.id = c.animal_id
             WHERE a.active = true AND a.gender = 'female'
             AND NOT EXISTS (
                 SELECT 1 FROM calvings c2 WHERE c2.animal_id = c.animal_id
                 AND c2.calving_date > c.calving_date
             )
             ORDER BY a.name",
        )
        .fetch_all(pool)
        .await
    }
    .map_err(AppError::Database)?;

    if rows.is_empty() {
        return Ok(Vec::new());
    }

    let animal_ids: Vec<i32> = rows.iter().map(|(id, _, _, _, _)| *id).collect();
    let calving_map: std::collections::HashMap<i32, chrono::NaiveDate> = rows
        .iter()
        .filter_map(|(id, _, _, _, calving_str)| {
            chrono::NaiveDate::parse_from_str(calving_str, "%Y-%m-%d")
                .ok()
                .map(|d| (*id, d))
        })
        .collect();

    let earliest = calving_map
        .values()
        .min()
        .copied()
        .unwrap_or(chrono::Utc::now().date_naive());

    let all_milk: Vec<(i32, i32, Option<f64>)> = sqlx::query_as(
        "SELECT m.animal_id, (m.date - $1::date)::int as dim, m.milk_amount
         FROM milk_day_productions m
         WHERE m.animal_id = ANY($2) AND m.date >= $1 AND m.date < $1 + INTERVAL '400 days'
         ORDER BY m.animal_id, dim",
    )
    .bind(earliest)
    .bind(&animal_ids)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    let mut milk_by_animal: std::collections::HashMap<i32, Vec<(i32, Option<f64>)>> =
        std::collections::HashMap::new();
    for (aid, dim, milk) in all_milk {
        milk_by_animal.entry(aid).or_default().push((dim, milk));
    }

    let today = chrono::Utc::now().date_naive();
    let mut results = Vec::new();

    for (aid, name, ln, lac, calving_str) in rows {
        let calving = match calving_map.get(&aid) {
            Some(&d) => d,
            None => continue,
        };
        let current_dim = (today - calving).num_days() as i32;
        if current_dim < 5 {
            continue;
        }

        let milk_rows = milk_by_animal.remove(&aid).unwrap_or_default();

        if milk_rows.len() < 3 {
            continue;
        }

        let has_enough_for_fit = milk_rows.len() >= 5;

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

        let mut fitted_curve = Vec::new();
        let mut forecast = Vec::new();
        let mut peak_milk = 0.0_f64;
        let mut peak_dim = 0_i32;
        let mut predicted_305: Option<f64> = None;

        if has_enough_for_fit {
            let (a, b, c) = fit_wood_model(&actual_map);

            let max_dim = (current_dim + 120).min(400);

            for dim in 1..=max_dim {
                let val = wood_predict(a, b, c, dim);
                if val > peak_milk {
                    peak_milk = val;
                    peak_dim = dim;
                }
                let pt = LactationPoint {
                    dim,
                    milk: Some(round2(val)),
                };
                if dim <= current_dim {
                    fitted_curve.push(pt);
                } else {
                    forecast.push(pt);
                }
            }

            predicted_305 = Some(
                (1..=305)
                    .map(|d| wood_predict(a, b, c, d))
                    .sum::<f64>(),
            );
        } else {
            peak_milk = actual_points
                .iter()
                .filter_map(|p| p.milk)
                .fold(0.0_f64, f64::max);
            peak_dim = actual_points
                .iter()
                .filter_map(|p| if p.milk == Some(peak_milk) { Some(p.dim) } else { None })
                .next()
                .unwrap_or(0);
        }

        results.push(LactationCurveResponse {
            animal_id: aid,
            animal_name: name,
            life_number: ln,
            lac_number: lac,
            calving_date: calving_str,
            current_dim,
            peak_milk: if has_enough_for_fit || peak_milk > 0.0 {
                Some(round2(peak_milk))
            } else {
                None
            },
            peak_dim: if peak_dim > 0 { Some(peak_dim) } else { None },
            predicted_total_305d: predicted_305.map(round2),
            actual_points,
            fitted_curve,
            forecast,
        });
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
            milk_deviation_zscore: milk_z.map(|z| round2(z)),
            rumination_deviation_zscore: rum_z.map(|z| round2(z)),
            activity_deviation_zscore: act_z.map(|z| round2(z)),
            scc_deviation_zscore: scc_z.map(|z| round2(z)),
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
            activity_signal: act_signal.map(|v| round2(v)),
            rumination_signal: rum_signal.map(|v| round2(v)),
            milk_signal: milk_signal.map(|v| round2(v)),
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
            avg_daily_milk: avg_milk.map(|v| round2(v)),
            avg_daily_feed: avg_feed.map(|v| round2(v)),
            estimated_milk_revenue_day: milk_rev.map(|v| round2(v)),
            estimated_feed_cost_day: feed_cost.map(|v| round2(v)),
            estimated_margin_day: margin_day.map(|v| round2(v)),
            margin_30d: margin_30d.map(|v| round2(v)),
            feed_cost_ratio: feed_ratio.map(|v| round2(v)),
        });
    }

    let herd_avg = if margin_count > 0 {
        Some(round2(total_margin / margin_count as f64))
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
                    (Some(avg), Some(oa)) if oa > 0.0 => Some(round2(avg / oa)),
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
        trend_7d: trend_7d.map(|v: f64| round2(v)),
        trend_30d: trend_30d.map(|v: f64| round2(v)),
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
            risk_score: round2(score),
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
            risk_score: round2(risk),
            risk_factors: factors,
        });
    }

    cows.sort_by(|a, b| b.risk_score.partial_cmp(&a.risk_score).unwrap_or(std::cmp::Ordering::Equal));
    Ok(CullingSurvivalResponse {
        cows,
        model_version: "rule-based-v1".to_string(),
    })
}

pub async fn energy_balance(pool: &PgPool) -> Result<EnergyBalanceResponse, AppError> {
    let rows: Vec<(i32, Option<String>, Option<String>, Option<f64>, Option<f64>, Option<f64>)> = sqlx::query_as(
        "SELECT a.id, a.name, a.life_number,
                recent_7d.fat as fat_7d,
                recent_7d.protein as protein_7d,
                recent_30d.fpr_trend as trend_30d
         FROM animals a
         LEFT JOIN LATERAL (
             SELECT AVG(q.fat_percentage)::float8 as fat,
                    AVG(q.protein_percentage)::float8 as protein
             FROM milk_quality q
             WHERE q.animal_id = a.id AND q.date >= CURRENT_DATE - INTERVAL '7 days'
         ) recent_7d ON true
         LEFT JOIN LATERAL (
             SELECT (recent.fpr / NULLIF(baseline.fpr, 0) - 1)::float8 as fpr_trend
             FROM (
                 SELECT AVG(q.fat_percentage)::float8 / NULLIF(AVG(q.protein_percentage)::float8, 0) as fpr
                 FROM milk_quality q WHERE q.animal_id = a.id AND q.date >= CURRENT_DATE - INTERVAL '7 days'
             ) recent,
             (
                 SELECT AVG(q.fat_percentage)::float8 / NULLIF(AVG(q.protein_percentage)::float8, 0) as fpr
                 FROM milk_quality q WHERE q.animal_id = a.id AND q.date >= CURRENT_DATE - INTERVAL '30 days'
                 AND q.date < CURRENT_DATE - INTERVAL '7 days'
             ) baseline
         ) recent_30d ON true
         WHERE a.active = true AND a.gender = 'female'
         ORDER BY a.name"
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    let cows: Vec<CowEnergyBalance> = rows
        .into_iter()
        .filter_map(|(id, name, ln, fat, protein, trend_30d)| {
            let fpr = match (fat, protein) {
                (Some(f), Some(p)) if p > 0.0 => Some(round2(f / p)),
                _ => None,
            };
            if fpr.is_none() && trend_30d.is_none() {
                return None;
            }

            let status = match fpr {
                Some(r) if r < 1.0 => "ketosis_risk",
                Some(r) if r > 1.5 => "acidosis_risk",
                Some(r) if r >= 1.2 && r <= 1.4 => "optimal",
                Some(_) => "normal",
                None => "unknown",
            };

            let trend_7d = fpr;

            Some(CowEnergyBalance {
                animal_id: id,
                animal_name: name,
                life_number: ln,
                avg_fat_pct: fat.map(|v| round2(v)),
                avg_protein_pct: protein.map(|v| round2(v)),
                fat_protein_ratio: fpr,
                status: status.to_string(),
                trend_7d,
                trend_30d: trend_30d.map(|v| round2(v)),
            })
        })
        .collect();

    Ok(EnergyBalanceResponse { cows })
}

pub async fn quarter_health(pool: &PgPool) -> Result<QuarterHealthResponse, AppError> {
    let rows: Vec<(i32, Option<String>, Option<String>, Option<f64>, Option<f64>, Option<f64>, Option<f64>)> = sqlx::query_as(
        "SELECT a.id, a.name, a.life_number,
                q_avg.lf as lf_cond,
                q_avg.lr as lr_cond,
                q_avg.rf as rf_cond,
                q_avg.rr as rr_cond
         FROM animals a
         LEFT JOIN LATERAL (
             SELECT AVG(v.lf_conductivity)::float8 as lf,
                    AVG(v.lr_conductivity)::float8 as lr,
                    AVG(v.rf_conductivity)::float8 as rf,
                    AVG(v.rr_conductivity)::float8 as rr
             FROM milk_visit_quality v
             WHERE v.animal_id = a.id AND v.visit_datetime >= CURRENT_DATE - INTERVAL '7 days'
         ) q_avg ON true
         WHERE a.active = true AND a.gender = 'female'
         ORDER BY a.name"
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    let cows: Vec<CowQuarterHealth> = rows
        .into_iter()
        .filter_map(|(id, name, ln, lf, lr, rf, rr)| {
            let values = [lf?, lr?, rf?, rr?];
            let names = ["LF", "LR", "RF", "RR"];
            let avg = (values[0] + values[1] + values[2] + values[3]) / 4.0;

            let mut max_asym = 0.0_f64;
            let mut worst = 0;
            for i in 0..4 {
                let asym = (values[i] - avg).abs();
                if asym > max_asym {
                    max_asym = asym;
                    worst = i;
                }
            }

            let risk_level = if max_asym > 10.0 {
                "high"
            } else if max_asym > 5.0 {
                "medium"
            } else if avg > 55.0 {
                "elevated"
            } else {
                "low"
            };

            Some(CowQuarterHealth {
                animal_id: id,
                animal_name: name,
                life_number: ln,
                lf_conductivity: Some(round2(values[0])),
                lr_conductivity: Some(round2(values[1])),
                rf_conductivity: Some(round2(values[2])),
                rr_conductivity: Some(round2(values[3])),
                avg_conductivity: Some(round2(avg)),
                max_asymmetry: Some(round2(max_asym)),
                worst_quarter: Some(names[worst].to_string()),
                risk_level: risk_level.to_string(),
            })
        })
        .collect();

    Ok(QuarterHealthResponse { cows })
}

pub async fn feed_efficiency(pool: &PgPool) -> Result<FeedEfficiencyResponse, AppError> {
    let rows: Vec<(i32, Option<String>, Option<String>, Option<f64>, Option<f64>)> = sqlx::query_as(
        "SELECT a.id, a.name, a.life_number,
                m.milk, f.feed
         FROM animals a
         LEFT JOIN LATERAL (
             SELECT AVG(m.milk_amount)::float8 as milk
             FROM milk_day_productions m WHERE m.animal_id = a.id AND m.date >= CURRENT_DATE - INTERVAL '30 days'
         ) m ON true
         LEFT JOIN LATERAL (
             SELECT AVG(f.total)::float8 as feed
             FROM feed_day_amounts f WHERE f.animal_id = a.id AND f.feed_date >= CURRENT_DATE - INTERVAL '30 days'
         ) f ON true
         WHERE a.active = true AND a.gender = 'female' AND m.milk IS NOT NULL
         ORDER BY m.milk DESC NULLS LAST"
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    let mut cows: Vec<CowFeedEfficiency> = rows
        .into_iter()
        .map(|(id, name, ln, milk, feed)| {
            let eff = match (milk, feed) {
                (Some(m), Some(f)) if f > 0.0 => Some(m / f),
                _ => None,
            };
            let mpf = match (milk, feed) {
                (Some(m), Some(f)) if f > 0.0 => Some(m / f),
                _ => None,
            };
            CowFeedEfficiency {
                animal_id: id,
                animal_name: name,
                life_number: ln,
                avg_daily_milk: milk,
                avg_daily_feed: feed,
                feed_efficiency: eff,
                milk_per_feed: mpf,
                efficiency_rank: None,
            }
        })
        .collect();

    let herd_avg = if !cows.is_empty() {
        let sum: f64 = cows.iter().filter_map(|c| c.feed_efficiency).sum();
        let cnt = cows.iter().filter(|c| c.feed_efficiency.is_some()).count();
        if cnt > 0 { Some(sum / cnt as f64) } else { None }
    } else {
        None
    };

    cows.sort_by(|a, b| b.feed_efficiency.partial_cmp(&a.feed_efficiency).unwrap_or(std::cmp::Ordering::Equal));
    for (i, cow) in cows.iter_mut().enumerate() {
        cow.efficiency_rank = Some((i + 1) as i32);
    }

    Ok(FeedEfficiencyResponse { cows, herd_avg_efficiency: herd_avg })
}

pub async fn dry_off_optimizer(pool: &PgPool) -> Result<DryOffOptimizerResponse, AppError> {
    let rows: Vec<(i32, Option<String>, Option<String>, Option<String>, Option<f64>, Option<f64>)> = sqlx::query_as(
        "SELECT a.id, a.name, a.life_number,
                p_duedate.exp_calving::text,
                m7.milk,
                scc.avg_scc
         FROM animals a
         JOIN LATERAL (
             SELECT c.calving_date + INTERVAL '280 days' as exp_calving
             FROM calvings c
             JOIN LATERAL (
                 SELECT i.insemination_date FROM inseminations i
                 WHERE i.animal_id = c.animal_id AND i.insemination_date > c.calving_date
                 ORDER BY i.insemination_date DESC LIMIT 1
             ) ai ON true
             JOIN pregnancies p ON p.animal_id = c.animal_id AND p.insemination_date = ai.insemination_date
             WHERE c.animal_id = a.id
             ORDER BY c.calving_date DESC LIMIT 1
         ) p_duedate ON true
         LEFT JOIN LATERAL (
             SELECT AVG(m.milk_amount)::float8 as milk FROM milk_day_productions m
             WHERE m.animal_id = a.id AND m.date >= CURRENT_DATE - INTERVAL '7 days'
         ) m7 ON true
         LEFT JOIN LATERAL (
             SELECT AVG(q.scc)::float8 as avg_scc FROM milk_quality q
             WHERE q.animal_id = a.id AND q.date >= CURRENT_DATE - INTERVAL '30 days'
         ) scc ON true
         WHERE a.active = true AND a.gender = 'female'
           AND p_duedate.exp_calving > CURRENT_DATE AND p_duedate.exp_calving < CURRENT_DATE + INTERVAL '120 days'
         ORDER BY p_duedate.exp_calving"
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    let cows: Vec<DryOffRecommendationEntry> = rows
        .into_iter()
        .filter_map(|(id, name, ln, exp_calving, milk, scc)| {
            let exp = exp_calving?;
            let exp_date = chrono::NaiveDate::parse_from_str(&exp, "%Y-%m-%d").ok()?;
            let dry_off = exp_date - chrono::Duration::days(60);
            let days_until = (dry_off - chrono::Local::now().date_naive()).num_days();

            let scc_status = match scc {
                Some(s) if s > 400000.0 => "elevated",
                Some(s) if s > 200000.0 => "moderate",
                _ => "normal",
            };

            let readiness = if days_until <= 0 {
                "overdue"
            } else if days_until <= 7 {
                "now"
            } else if days_until <= 21 {
                "soon"
            } else {
                "monitor"
            };

            Some(DryOffRecommendationEntry {
                animal_id: id,
                animal_name: name,
                life_number: ln,
                expected_calving_date: Some(exp),
                current_daily_milk: milk,
                recommended_dry_off_date: Some(dry_off.to_string()),
                days_until_dry_off: Some(days_until),
                scc_status: scc_status.to_string(),
                readiness: readiness.to_string(),
            })
        })
        .collect();

    Ok(DryOffOptimizerResponse { cows })
}

pub async fn lifetime_value(pool: &PgPool) -> Result<LifetimeValueResponse, AppError> {
    let rows: Vec<(i32, Option<String>, Option<String>, Option<f64>, i64, Option<f64>, Option<f64>)> = sqlx::query_as(
        "SELECT a.id, a.name, a.life_number,
                EXTRACT(YEAR FROM AGE(CURRENT_DATE, a.birth_date))::float8 as age_years,
                COALESCE(lac_cnt.n, 0) as lac_count,
                total_milk.milk as total_milk,
                avg_milk.avg_m as avg_milk_per_lac
         FROM animals a
         LEFT JOIN LATERAL (SELECT COUNT(*)::int8 as n FROM calvings c WHERE c.animal_id = a.id) lac_cnt ON true
         LEFT JOIN LATERAL (
             SELECT SUM(m.milk_amount)::float8 as milk FROM milk_day_productions m WHERE m.animal_id = a.id
         ) total_milk ON true
         LEFT JOIN LATERAL (
             SELECT AVG(lac_milk.total)::float8 as avg_m
             FROM (
                 SELECT SUM(m.milk_amount)::float8 as total
                 FROM milk_day_productions m
                 JOIN calvings c ON c.animal_id = m.animal_id
                 WHERE m.animal_id = a.id AND m.date >= c.calving_date
                 GROUP BY c.id
             ) lac_milk
         ) avg_milk ON true
         WHERE a.active = true AND a.gender = 'female'
         ORDER BY a.name"
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    let milk_price = 25.0_f64;
    let feed_cost_per_day = 150.0_f64;
    let lactation_days = 305.0_f64;

    let cows: Vec<LifetimeValueEntry> = rows
        .into_iter()
        .map(|(id, name, ln, age, lac_count, total_milk, avg_milk_per_lac)| {
            let max_lacs: i32 = 6;
            let remaining = max_lacs.saturating_sub(lac_count as i32).max(0);

            let avg_per_lac = avg_milk_per_lac.unwrap_or(0.0);
            let proj_milk_val = remaining as f64 * avg_per_lac * milk_price;
            let proj_feed = remaining as f64 * lactation_days * feed_cost_per_day;
            let net = proj_milk_val - proj_feed;

            let recommendation = if remaining <= 0 || age.unwrap_or(0.0) > 8.0 {
                "culling_candidate"
            } else if net < 0.0 {
                "review"
            } else if remaining <= 1 {
                "last_lactation"
            } else {
                "keep"
            };

            LifetimeValueEntry {
                animal_id: id,
                animal_name: name,
                life_number: ln,
                age_years: age,
                lactation_count: lac_count,
                total_milk_produced: total_milk,
                estimated_remaining_lactations: remaining,
                projected_milk_value: Some(round2(proj_milk_val)),
                projected_feed_cost: Some(round2(proj_feed)),
                projected_net_value: Some(round2(net)),
                recommendation: recommendation.to_string(),
            }
        })
        .collect();

    Ok(LifetimeValueResponse { cows })
}

pub async fn estrus_detection(pool: &PgPool) -> Result<EstrusResponse, AppError> {
    let rows: Vec<(i32, Option<String>, Option<i64>, Option<f64>, Option<f64>, Option<f64>)> = sqlx::query_as(
        "SELECT a.id, a.name,
                (CURRENT_DATE - c.calving_date)::int8 as dim,
                act_ratio.ratio as activity_signal,
                rum_ratio.ratio as rumination_signal,
                milk_ratio.ratio as milk_signal
         FROM animals a
         LEFT JOIN LATERAL (
             SELECT calving_date FROM calvings c WHERE c.animal_id = a.id ORDER BY c.calving_date DESC LIMIT 1
         ) c ON true
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
             AND NOT EXISTS (SELECT 1 FROM calvings cc WHERE cc.animal_id = a.id AND cc.calving_date > p.pregnancy_date)
         )"
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    let mut predictions = Vec::new();
    for (id, name, dim, act_signal, rum_signal, milk_signal) in rows {
        let in_window = dim.map_or(false, |d| d >= 30 && d <= 150);

        let mut score = 0.0_f64;
        let mut signals = Vec::new();

        if let Some(s) = act_signal {
            if s > 1.4 {
                score += 0.45;
                signals.push("high_activity".to_string());
            } else if s > 1.2 {
                score += 0.25;
                signals.push("elevated_activity".to_string());
            }
        }

        if let Some(s) = rum_signal {
            if s < 0.8 {
                score += 0.35;
                signals.push("low_rumination".to_string());
            } else if s < 0.9 {
                score += 0.15;
                signals.push("reduced_rumination".to_string());
            }
        }

        if let Some(s) = milk_signal {
            if s < 0.85 {
                score += 0.2;
                signals.push("milk_drop".to_string());
            }
        }

        if in_window {
            score += 0.1;
        } else {
            score *= 0.5;
        }

        if score < 0.15 {
            continue;
        }

        score = score.min(1.0);

        let status = if score >= 0.7 {
            "in_estrus"
        } else if score >= 0.4 {
            "approaching"
        } else {
            "possible"
        };

        let window = if in_window {
            Some("within_breeding_window".to_string())
        } else {
            None
        };

        predictions.push(EstrusPrediction {
            animal_id: id,
            animal_name: name,
            estrus_probability: round2(score),
            status: status.to_string(),
            contributing_signals: signals,
            optimal_window: window,
            model_version: "rule-based-v1".to_string(),
        });
    }

    predictions.sort_by(|a, b| b.estrus_probability.partial_cmp(&a.estrus_probability).unwrap_or(std::cmp::Ordering::Equal));
    Ok(EstrusResponse { predictions })
}

pub async fn equipment_anomaly(pool: &PgPool) -> Result<EquipmentAnomalyResponse, AppError> {
    let rows: Vec<(i32, Option<String>, Option<f64>, Option<f64>, Option<f64>, Option<f64>, Option<f64>, Option<i32>)> = sqlx::query_as(
        "SELECT a.id, a.name,
                v_avg.cond as avg_conductivity,
                v_avg.temp as avg_temperature,
                v_avg.speed as avg_milk_speed,
                cond_dev.dev as conductivity_deviation,
                speed_dev.dev as speed_deviation,
                d.device_address
         FROM animals a
         LEFT JOIN LATERAL (
             SELECT AVG((v.lf_conductivity + v.lr_conductivity + v.rf_conductivity + v.rr_conductivity) / 4.0)::float8 as cond,
                    AVG(v.milk_temperature)::float8 as temp,
                    AVG(v.milk_speed)::float8 as speed
             FROM milk_visit_quality v
             WHERE v.animal_id = a.id AND v.visit_datetime >= CURRENT_DATE - INTERVAL '7 days'
         ) v_avg ON true
         LEFT JOIN LATERAL (
             SELECT (recent.cond / NULLIF(baseline.cond, 0) - 1)::float8 as dev FROM (
                 SELECT AVG((lf_conductivity + lr_conductivity + rf_conductivity + rr_conductivity) / 4.0)::float8 as cond
                 FROM milk_visit_quality WHERE animal_id = a.id AND visit_datetime >= CURRENT_DATE - INTERVAL '3 days'
             ) recent, (
                 SELECT AVG((lf_conductivity + lr_conductivity + rf_conductivity + rr_conductivity) / 4.0)::float8 as cond
                 FROM milk_visit_quality WHERE animal_id = a.id AND visit_datetime >= CURRENT_DATE - INTERVAL '30 days'
                 AND visit_datetime < CURRENT_DATE - INTERVAL '3 days'
             ) baseline
         ) cond_dev ON true
         LEFT JOIN LATERAL (
             SELECT (recent.speed / NULLIF(baseline.speed, 0) - 1)::float8 as dev FROM (
                 SELECT AVG(milk_speed)::float8 as speed
                 FROM milk_visit_quality WHERE animal_id = a.id AND visit_datetime >= CURRENT_DATE - INTERVAL '3 days'
             ) recent, (
                 SELECT AVG(milk_speed)::float8 as speed
                 FROM milk_visit_quality WHERE animal_id = a.id AND visit_datetime >= CURRENT_DATE - INTERVAL '30 days'
                 AND visit_datetime < CURRENT_DATE - INTERVAL '3 days'
             ) baseline
         ) speed_dev ON true
         LEFT JOIN LATERAL (
             SELECT device_address FROM devices WHERE animal_id = a.id LIMIT 1
         ) d ON true
         WHERE a.active = true AND a.gender = 'female'
         AND v_avg.cond IS NOT NULL"
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    let mut entries = Vec::new();
    for (id, name, avg_cond, avg_temp, _avg_speed, cond_dev, speed_dev, device) in rows {
        let mut score = 0.0_f64;
        let mut flags = Vec::new();

        if let Some(c) = avg_cond {
            if c > 60.0 {
                score += 0.4;
                flags.push("high_conductivity".to_string());
            } else if c > 50.0 {
                score += 0.15;
                flags.push("elevated_conductivity".to_string());
            }
        }

        if let Some(dev) = cond_dev {
            if dev > 0.2 {
                score += 0.3;
                flags.push("conductivity_spike".to_string());
            } else if dev > 0.1 {
                score += 0.1;
                flags.push("conductivity_rising".to_string());
            }
        }

        if let Some(t) = avg_temp {
            if t > 40.0 {
                score += 0.25;
                flags.push("high_temperature".to_string());
            }
        }

        if let Some(dev) = speed_dev {
            if dev < -0.3 {
                score += 0.2;
                flags.push("slow_milk_speed".to_string());
            }
        }

        let is_anomaly = score >= 0.4;
        let severity = if score >= 0.7 {
            "critical"
        } else if score >= 0.4 {
            "warning"
        } else {
            "normal"
        };

        entries.push(EquipmentAnomalyEntry {
            animal_id: id,
            animal_name: name,
            is_anomaly,
            anomaly_score: round2(score),
            severity: severity.to_string(),
            flags,
            device_address: device,
            model_version: "rule-based-v1".to_string(),
        });
    }

    entries.sort_by(|a, b| b.anomaly_score.partial_cmp(&a.anomaly_score).unwrap_or(std::cmp::Ordering::Equal));
    Ok(EquipmentAnomalyResponse { entries })
}

pub async fn feed_recommendation(pool: &PgPool) -> Result<FeedRecommendationResponse, AppError> {
    let rows: Vec<(i32, Option<String>, Option<f64>, Option<f64>, Option<i64>, Option<i64>)> = sqlx::query_as(
        "SELECT a.id, a.name,
                f.feed as current_feed_avg,
                m.milk as avg_milk_30d,
                dim.days as dim_days,
                lac.n as lactation_number
         FROM animals a
         LEFT JOIN LATERAL (
             SELECT AVG(f.total)::float8 as feed FROM feed_day_amounts f
             WHERE f.animal_id = a.id AND f.feed_date >= CURRENT_DATE - INTERVAL '7 days'
         ) f ON true
         LEFT JOIN LATERAL (
             SELECT AVG(m.milk_amount)::float8 as milk FROM milk_day_productions m
             WHERE m.animal_id = a.id AND m.date >= CURRENT_DATE - INTERVAL '30 days'
         ) m ON true
         LEFT JOIN LATERAL (
             SELECT (CURRENT_DATE - c.calving_date)::int8 as days
             FROM calvings c WHERE c.animal_id = a.id ORDER BY c.calving_date DESC LIMIT 1
         ) dim ON true
         LEFT JOIN LATERAL (
             SELECT COUNT(*)::int8 as n FROM calvings c WHERE c.animal_id = a.id
         ) lac ON true
         WHERE a.active = true AND a.gender = 'female'
         AND f.feed IS NOT NULL"
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    let mut recommendations = Vec::new();
    for (id, name, feed_avg, milk_avg, dim_days, lac_count) in rows {
        let current_feed = feed_avg.unwrap_or(0.0);
        let milk = milk_avg.unwrap_or(0.0);
        let dim = dim_days.unwrap_or(150);
        let lac_n = lac_count.unwrap_or(1);

        let base_feed = 12.0 + milk * 0.4;

        let dim_factor = if dim < 60 {
            1.15
        } else if dim < 120 {
            1.05
        } else if dim > 250 {
            0.9
        } else {
            1.0
        };

        let lac_factor = if lac_n <= 1 {
            1.0
        } else if lac_n <= 3 {
            1.05
        } else {
            1.1
        };

        let recommended = base_feed * dim_factor * lac_factor;
        let diff = recommended - current_feed;

        let suggestion = if diff > 2.0 {
            "increase_feed".to_string()
        } else if diff < -2.0 {
            "reduce_feed".to_string()
        } else {
            "maintain".to_string()
        };

        recommendations.push(FeedRecommendationEntry {
            animal_id: id,
            animal_name: name,
            current_feed_avg: round2(current_feed),
            recommended_feed: round2(recommended),
            difference_kg: round2(diff),
            suggestion,
            dim_days: dim as i32,
            lactation_number: lac_n as i32,
            model_version: "rule-based-v1".to_string(),
        });
    }

    Ok(FeedRecommendationResponse { recommendations })
}

pub async fn ketosis_warning(pool: &PgPool) -> Result<KetosisWarningResponse, AppError> {
    let rows: Vec<(i32, Option<String>, Option<f64>, Option<f64>, Option<f64>, Option<i64>, Option<f64>)> = sqlx::query_as(
        "SELECT a.id, a.name,
                recent.fpr as fpr_7d,
                baseline.fpr as fpr_30d,
                fpr_trend.trend as fpr_trend,
                dim.days as dim_days,
                rum.rum as rum_7d
         FROM animals a
         LEFT JOIN LATERAL (
             SELECT AVG(q.fat_percentage)::float8 / NULLIF(AVG(q.protein_percentage)::float8, 0) as fpr
             FROM milk_quality q WHERE q.animal_id = a.id AND q.date >= CURRENT_DATE - INTERVAL '7 days'
         ) recent ON true
         LEFT JOIN LATERAL (
             SELECT AVG(q.fat_percentage)::float8 / NULLIF(AVG(q.protein_percentage)::float8, 0) as fpr
             FROM milk_quality q WHERE q.animal_id = a.id AND q.date >= CURRENT_DATE - INTERVAL '30 days'
             AND q.date < CURRENT_DATE - INTERVAL '7 days'
         ) baseline ON true
         LEFT JOIN LATERAL (
             SELECT (recent.fpr / NULLIF(baseline.fpr, 0) - 1)::float8 as trend FROM (
                 SELECT AVG(q.fat_percentage)::float8 / NULLIF(AVG(q.protein_percentage)::float8, 0) as fpr
                 FROM milk_quality q WHERE q.animal_id = a.id AND q.date >= CURRENT_DATE - INTERVAL '7 days'
             ) recent, (
                 SELECT AVG(q.fat_percentage)::float8 / NULLIF(AVG(q.protein_percentage)::float8, 0) as fpr
                 FROM milk_quality q WHERE q.animal_id = a.id AND q.date >= CURRENT_DATE - INTERVAL '30 days'
                 AND q.date < CURRENT_DATE - INTERVAL '7 days'
             ) baseline
         ) fpr_trend ON true
         LEFT JOIN LATERAL (
             SELECT (CURRENT_DATE - c.calving_date)::int8 as days
             FROM calvings c WHERE c.animal_id = a.id ORDER BY c.calving_date DESC LIMIT 1
         ) dim ON true
         LEFT JOIN LATERAL (
             SELECT AVG(r.rumination_minutes)::float8 as rum FROM ruminations r
             WHERE r.animal_id = a.id AND r.date >= CURRENT_DATE - INTERVAL '7 days'
         ) rum ON true
         WHERE a.active = true AND a.gender = 'female'
         AND recent.fpr IS NOT NULL"
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    let mut predictions = Vec::new();
    for (id, name, fpr_7d, _fpr_30d, fpr_trend, dim_days, rum_7d) in rows {
        let fpr = match fpr_7d {
            Some(f) => f,
            None => continue,
        };

        let mut risk = 0.0_f64;
        let mut factors = Vec::new();
        let mut risk_type = "subclinical".to_string();

        if fpr < 1.0 {
            risk += 0.5;
            factors.push("fpr_below_1.0".to_string());
            risk_type = "clinical".to_string();
        } else if fpr < 1.1 {
            risk += 0.25;
            factors.push("fpr_low".to_string());
        }

        if let Some(trend) = fpr_trend {
            if trend < -0.1 {
                risk += 0.3;
                factors.push("fpr_declining".to_string());
            }
        }

        if let Some(dim) = dim_days {
            if dim < 30 {
                risk += 0.25;
                factors.push("early_lactation".to_string());
            } else if dim < 60 {
                risk += 0.1;
                factors.push("fresh cow".to_string());
            }
        }

        if let Some(rum) = rum_7d {
            if rum < 400.0 {
                risk += 0.2;
                factors.push("low_rumination".to_string());
            }
        }

        if risk < 0.1 {
            continue;
        }

        risk = risk.min(1.0);

        let severity = if risk >= 0.6 {
            "high"
        } else if risk >= 0.3 {
            "moderate"
        } else {
            "low"
        };

        predictions.push(KetosisWarningEntry {
            animal_id: id,
            animal_name: name,
            risk_probability: round2(risk),
            risk_type,
            severity: severity.to_string(),
            fpr_current: round2(fpr),
            fpr_trend: fpr_trend.unwrap_or(0.0),
            contributing_factors: factors,
            model_version: "rule-based-v1".to_string(),
        });
    }

    predictions.sort_by(|a, b| b.risk_probability.partial_cmp(&a.risk_probability).unwrap_or(std::cmp::Ordering::Equal));
    Ok(KetosisWarningResponse { predictions })
}

pub async fn animal_summary(
    pool: &PgPool,
    animal_id: i32,
) -> Result<AnimalSummaryResponse, AppError> {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM animals WHERE id = $1 AND active = true AND gender = 'female')",
    )
    .bind(animal_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)?;

    if !exists {
        return Ok(AnimalSummaryResponse {
            animal_id,
            health_index: None,
            mastitis_risk: None,
            estrus: None,
            energy_balance: None,
            feed_recommendation: None,
            ketosis_warning: None,
            lifetime_value: None,
            culling_risk: None,
            cluster: None,
        });
    }

    let (health, mastitis, estrus, energy, feed, ketosis, lifetime, culling, cluster) = tokio::try_join!(
        summary_health_index(pool, animal_id),
        summary_mastitis_risk(pool, animal_id),
        summary_estrus(pool, animal_id),
        summary_energy_balance(pool, animal_id),
        summary_feed_rec(pool, animal_id),
        summary_ketosis(pool, animal_id),
        summary_lifetime(pool, animal_id),
        summary_culling(pool, animal_id),
        summary_cluster(pool, animal_id),
    )?;

    Ok(AnimalSummaryResponse {
        animal_id,
        health_index: health,
        mastitis_risk: mastitis,
        estrus,
        energy_balance: energy,
        feed_recommendation: feed,
        ketosis_warning: ketosis,
        lifetime_value: lifetime,
        culling_risk: culling,
        cluster,
    })
}

async fn summary_health_index(pool: &PgPool, id: i32) -> Result<Option<CowHealthIndex>, AppError> {
    let rows: Vec<(i32, Option<String>, Option<String>, Option<f64>, Option<f64>, Option<f64>, Option<f64>, Option<f64>, Option<f64>, Option<f64>, Option<f64>, Option<f64>)> = sqlx::query_as(
        "SELECT a.id, a.name, a.life_number,
                short_m.milk, long_m.milk, long_m.std as milk_std,
                short_r.rum, long_r.rum, long_r.std as rum_std,
                short_a.act, long_a.act, long_a.std as act_std
         FROM animals a
         LEFT JOIN LATERAL (
             SELECT AVG(m.milk_amount)::float8 as milk FROM milk_day_productions m WHERE m.animal_id = a.id AND m.date >= CURRENT_DATE - INTERVAL '3 days'
         ) short_m ON true
         LEFT JOIN LATERAL (
             SELECT AVG(m.milk_amount)::float8 as milk, STDDEV(m.milk_amount)::float8 as std FROM milk_day_productions m WHERE m.animal_id = a.id AND m.date >= CURRENT_DATE - INTERVAL '30 days'
         ) long_m ON true
         LEFT JOIN LATERAL (
             SELECT AVG(r.rumination_minutes)::float8 as rum, STDDEV(r.rumination_minutes)::float8 as std FROM ruminations r WHERE r.animal_id = a.id AND r.date >= CURRENT_DATE - INTERVAL '30 days' AND r.date < CURRENT_DATE - INTERVAL '3 days'
         ) short_r ON true
         LEFT JOIN LATERAL (
             SELECT AVG(r.rumination_minutes)::float8 as rum, STDDEV(r.rumination_minutes)::float8 as std FROM ruminations r WHERE r.animal_id = a.id AND r.date >= CURRENT_DATE - INTERVAL '30 days'
         ) long_r ON true
         LEFT JOIN LATERAL (
             SELECT AVG(act.activity_counter)::float8 as act FROM activities act WHERE act.animal_id = a.id AND act.activity_datetime >= CURRENT_DATE - INTERVAL '3 days'
         ) short_a ON true
         LEFT JOIN LATERAL (
             SELECT AVG(act.activity_counter)::float8 as act, STDDEV(act.activity_counter)::float8 as std FROM activities act WHERE act.animal_id = a.id AND act.activity_datetime >= CURRENT_DATE - INTERVAL '30 days' AND act.activity_datetime < CURRENT_DATE - INTERVAL '3 days'
         ) long_a ON true
         WHERE a.id = $1"
    )
    .bind(id)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    let Some((_, name, ln, short_milk, long_milk, milk_std, short_rum, long_rum, rum_std, short_act, long_act, act_std)) = rows.into_iter().next() else {
        return Ok(None);
    };

    let scc_row: Option<(f64, Option<f64>)> = sqlx::query_as(
        "SELECT recent.scc, baseline.avg_scc FROM
         (SELECT COALESCE(AVG(q.scc),0)::float8 as scc FROM milk_quality q WHERE q.animal_id = $1 AND q.date >= CURRENT_DATE - INTERVAL '3 days') recent,
         (SELECT AVG(q.scc)::float8 as avg_scc FROM milk_quality q WHERE q.animal_id = $1 AND q.date >= CURRENT_DATE - INTERVAL '90 days' AND q.date < CURRENT_DATE - INTERVAL '3 days') baseline"
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?;

    let milk_z = zscore(short_milk, long_milk, milk_std);
    let rum_z = zscore(short_rum, long_rum, rum_std);
    let act_z = zscore(short_act, long_act, act_std);

    let (scc_z, scc_concern) = if let Some((recent, avg)) = scc_row {
        if recent > 0.0 && avg.is_some() {
            let baseline = avg.unwrap();
            let ratio = recent / baseline;
            let z = if baseline > 0.0 { (recent - baseline) / (baseline * 0.5) } else { 0.0 };
            (Some(z), ratio > 1.5)
        } else {
            (None, false)
        }
    } else {
        (None, false)
    };

    let mut score = 100.0_f64;
    let mut concerns = Vec::new();

    if let Some(z) = milk_z {
        if z < -2.0 { score -= 30.0; concerns.push(("milk_drop".to_string(), z)); }
        else if z < -1.0 { score -= 15.0; concerns.push(("milk_drop".to_string(), z)); }
    }
    if let Some(z) = rum_z {
        if z < -2.0 { score -= 25.0; concerns.push(("rumination_drop".to_string(), z)); }
        else if z < -1.0 { score -= 10.0; concerns.push(("rumination_drop".to_string(), z)); }
    }
    if let Some(z) = act_z {
        if z < -2.0 { score -= 20.0; concerns.push(("activity_drop".to_string(), z)); }
        else if z < -1.0 { score -= 10.0; concerns.push(("activity_drop".to_string(), z)); }
    }
    if let Some(z) = scc_z {
        if z > 2.0 { score -= 25.0; concerns.push(("high_scc".to_string(), z)); }
        else if z > 1.0 || scc_concern { score -= 10.0; concerns.push(("high_scc".to_string(), z)); }
    }

    score = score.max(0.0).min(100.0);
    let risk_level = if score < 40.0 { "critical" } else if score < 60.0 { "high" } else if score < 80.0 { "moderate" } else { "low" };
    concerns.sort_by(|a, b| a.1.abs().partial_cmp(&b.1.abs()).unwrap_or(std::cmp::Ordering::Equal).reverse());

    Ok(Some(CowHealthIndex {
        animal_id: id,
        animal_name: name,
        life_number: ln,
        health_score: round2(score),
        milk_deviation_zscore: milk_z.map(round2),
        rumination_deviation_zscore: rum_z.map(round2),
        activity_deviation_zscore: act_z.map(round2),
        scc_deviation_zscore: scc_z.map(round2),
        risk_level: risk_level.to_string(),
        top_concern: concerns.first().map(|c| c.0.clone()),
    }))
}

async fn summary_mastitis_risk(pool: &PgPool, id: i32) -> Result<Option<MastitisRiskEntry>, AppError> {
    let row: Option<(Option<f64>, Option<f64>, Option<f64>, Option<f64>, Option<i64>)> = sqlx::query_as(
        "SELECT scc_latest.scc, scc_trend.ratio, cond.avg_cond, milk_dev.dev, dim.days
         FROM animals a
         LEFT JOIN LATERAL (SELECT AVG(q.scc)::float8 as scc FROM milk_quality q WHERE q.animal_id = a.id AND q.date >= CURRENT_DATE - INTERVAL '7 days') scc_latest ON true
         LEFT JOIN LATERAL (
             SELECT (recent.scc / NULLIF(baseline.scc, 0))::float8 as ratio FROM
             (SELECT AVG(q.scc)::float8 as scc FROM milk_quality q WHERE q.animal_id = a.id AND q.date >= CURRENT_DATE - INTERVAL '7 days') recent,
             (SELECT AVG(q.scc)::float8 as scc FROM milk_quality q WHERE q.animal_id = a.id AND q.date >= CURRENT_DATE - INTERVAL '90 days' AND q.date < CURRENT_DATE - INTERVAL '7 days') baseline
         ) scc_trend ON true
         LEFT JOIN LATERAL (SELECT AVG((v.lf_conductivity + v.lr_conductivity + v.rf_conductivity + v.rr_conductivity)::float8 / 4.0) as avg_cond FROM milk_visit_quality v WHERE v.animal_id = a.id AND v.visit_datetime >= CURRENT_DATE - INTERVAL '7 days') cond ON true
         LEFT JOIN LATERAL (
             SELECT (recent.milk / NULLIF(baseline.milk, 0) - 1)::float8 as dev FROM
             (SELECT AVG(m.milk_amount)::float8 as milk FROM milk_day_productions m WHERE m.animal_id = a.id AND m.date >= CURRENT_DATE - INTERVAL '7 days') recent,
             (SELECT AVG(m.milk_amount)::float8 as milk FROM milk_day_productions m WHERE m.animal_id = a.id AND m.date >= CURRENT_DATE - INTERVAL '30 days' AND m.date < CURRENT_DATE - INTERVAL '7 days') baseline
         ) milk_dev ON true
         LEFT JOIN LATERAL (SELECT (CURRENT_DATE - c.calving_date)::int8 as days FROM calvings c WHERE c.animal_id = a.id ORDER BY c.calving_date DESC LIMIT 1) dim ON true
         WHERE a.id = $1"
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?;

    let Some(row) = row else { return Ok(None) };
    let (scc, scc_t, cond, milk_dev, dim) = row;
    let Some(recent_scc) = scc else { return Ok(None) };

    let name: Option<String> = sqlx::query_scalar("SELECT name FROM animals WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)?;

    let mut score = 0.0_f64;
    let mut factors = Vec::new();
    if recent_scc > 500000.0 { score += 0.4; factors.push("SCC>500k".to_string()); }
    else if recent_scc > 300000.0 { score += 0.25; factors.push("SCC>300k".to_string()); }
    else if recent_scc > 200000.0 { score += 0.1; factors.push("SCC>200k".to_string()); }
    if let Some(t) = scc_t { if t > 2.0 { score += 0.25; factors.push("SCC↑↑".to_string()); } else if t > 1.5 { score += 0.15; factors.push("SCC↑".to_string()); } }
    if let Some(c) = cond { if c > 60.0 { score += 0.2; factors.push("conductivity↑".to_string()); } }
    if let Some(d) = milk_dev { if d < -0.15 { score += 0.15; factors.push("milk↓".to_string()); } }
    if let Some(d) = dim { if d < 30 { score += 0.1; factors.push("early_lactation".to_string()); } }

    if score < 0.05 { return Ok(None); }
    score = score.min(1.0);
    let risk_level = if score >= 0.6 { "high" } else if score >= 0.3 { "medium" } else { "low" };

    Ok(Some(MastitisRiskEntry {
        animal_id: id,
        animal_name: name,
        life_number: None,
        risk_score: round2(score),
        risk_level: risk_level.to_string(),
        contributing_factors: factors,
    }))
}

async fn summary_estrus(pool: &PgPool, id: i32) -> Result<Option<EstrusPrediction>, AppError> {
    let row: Option<(Option<i64>, Option<f64>, Option<f64>, Option<f64>)> = sqlx::query_as(
        "SELECT dim.days, act_ratio.ratio, rum_ratio.ratio, milk_ratio.ratio
         FROM animals a
         LEFT JOIN LATERAL (SELECT (CURRENT_DATE - c.calving_date)::int8 as days FROM calvings c WHERE c.animal_id = a.id ORDER BY c.calving_date DESC LIMIT 1) dim ON true
         LEFT JOIN LATERAL (
             SELECT (recent.act / NULLIF(baseline.act, 0))::float8 as ratio FROM
             (SELECT AVG(activity_counter)::float8 as act FROM activities WHERE animal_id = a.id AND activity_datetime >= CURRENT_DATE - INTERVAL '2 days') recent,
             (SELECT AVG(activity_counter)::float8 as act FROM activities WHERE animal_id = a.id AND activity_datetime >= CURRENT_DATE - INTERVAL '14 days' AND activity_datetime < CURRENT_DATE - INTERVAL '2 days') baseline
         ) act_ratio ON true
         LEFT JOIN LATERAL (
             SELECT (recent.rum / NULLIF(baseline.rum, 0))::float8 as ratio FROM
             (SELECT AVG(rumination_minutes)::float8 as rum FROM ruminations WHERE animal_id = a.id AND date >= CURRENT_DATE - INTERVAL '2 days') recent,
             (SELECT AVG(rumination_minutes)::float8 as rum FROM ruminations WHERE animal_id = a.id AND date >= CURRENT_DATE - INTERVAL '14 days' AND date < CURRENT_DATE - INTERVAL '2 days') baseline
         ) rum_ratio ON true
         LEFT JOIN LATERAL (
             SELECT (recent.milk / NULLIF(baseline.milk, 0))::float8 as ratio FROM
             (SELECT AVG(milk_amount)::float8 as milk FROM milk_day_productions WHERE animal_id = a.id AND date >= CURRENT_DATE - INTERVAL '2 days') recent,
             (SELECT AVG(milk_amount)::float8 as milk FROM milk_day_productions WHERE animal_id = a.id AND date >= CURRENT_DATE - INTERVAL '14 days' AND date < CURRENT_DATE - INTERVAL '2 days') baseline
         ) milk_ratio ON true
         WHERE a.id = $1"
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?;

    let Some((dim, act_signal, rum_signal, milk_signal)) = row else { return Ok(None) };

    let in_window = dim.map_or(false, |d| d >= 30 && d <= 150);
    let mut score = 0.0_f64;
    let mut signals = Vec::new();

    if let Some(s) = act_signal { if s > 1.4 { score += 0.45; signals.push("high_activity".to_string()); } else if s > 1.2 { score += 0.25; signals.push("elevated_activity".to_string()); } }
    if let Some(s) = rum_signal { if s < 0.8 { score += 0.35; signals.push("low_rumination".to_string()); } else if s < 0.9 { score += 0.15; signals.push("reduced_rumination".to_string()); } }
    if let Some(s) = milk_signal { if s < 0.85 { score += 0.2; signals.push("milk_drop".to_string()); } }
    if in_window { score += 0.1; } else { score *= 0.5; }
    if score < 0.15 { return Ok(None); }
    score = score.min(1.0);

    let status = if score >= 0.7 { "in_estrus" } else if score >= 0.4 { "approaching" } else { "possible" };
    let name: Option<String> = sqlx::query_scalar("SELECT name FROM animals WHERE id = $1").bind(id).fetch_optional(pool).await.map_err(AppError::Database)?;

    Ok(Some(EstrusPrediction {
        animal_id: id,
        animal_name: name,
        estrus_probability: round2(score),
        status: status.to_string(),
        contributing_signals: signals,
        optimal_window: if in_window { Some("within_breeding_window".to_string()) } else { None },
        model_version: "rule-based-v1".to_string(),
    }))
}

async fn summary_energy_balance(pool: &PgPool, id: i32) -> Result<Option<CowEnergyBalance>, AppError> {
    let row: Option<(Option<f64>, Option<f64>, Option<f64>)> = sqlx::query_as(
        "SELECT recent_7d.fat, recent_7d.protein, recent_30d.fpr_trend
         FROM animals a
         LEFT JOIN LATERAL (
             SELECT AVG(q.fat_percentage)::float8 as fat, AVG(q.protein_percentage)::float8 as protein FROM milk_quality q WHERE q.animal_id = a.id AND q.date >= CURRENT_DATE - INTERVAL '7 days'
         ) recent_7d ON true
         LEFT JOIN LATERAL (
             SELECT (recent.fpr / NULLIF(baseline.fpr, 0) - 1)::float8 as fpr_trend FROM
             (SELECT AVG(fat_percentage)::float8 / NULLIF(AVG(protein_percentage)::float8, 0) as fpr FROM milk_quality WHERE animal_id = a.id AND date >= CURRENT_DATE - INTERVAL '7 days') recent,
             (SELECT AVG(fat_percentage)::float8 / NULLIF(AVG(protein_percentage)::float8, 0) as fpr FROM milk_quality WHERE animal_id = a.id AND date >= CURRENT_DATE - INTERVAL '30 days' AND date < CURRENT_DATE - INTERVAL '7 days') baseline
         ) recent_30d ON true
         WHERE a.id = $1"
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?;

    let Some((fat, protein, trend_30d)) = row else { return Ok(None) };
    let fpr = match (fat, protein) { (Some(f), Some(p)) if p > 0.0 => Some(round2(f / p)), _ => None };
    if fpr.is_none() && trend_30d.is_none() { return Ok(None); }

    let status = match fpr {
        Some(r) if r < 1.0 => "ketosis_risk",
        Some(r) if r > 1.5 => "acidosis_risk",
        Some(r) if r >= 1.2 && r <= 1.4 => "optimal",
        Some(_) => "normal",
        None => "unknown",
    };
    let name: Option<String> = sqlx::query_scalar("SELECT name FROM animals WHERE id = $1").bind(id).fetch_optional(pool).await.map_err(AppError::Database)?;

    Ok(Some(CowEnergyBalance {
        animal_id: id,
        animal_name: name,
        life_number: None,
        avg_fat_pct: fat.map(round2),
        avg_protein_pct: protein.map(round2),
        fat_protein_ratio: fpr,
        status: status.to_string(),
        trend_7d: fpr,
        trend_30d: trend_30d.map(round2),
    }))
}

async fn summary_feed_rec(pool: &PgPool, id: i32) -> Result<Option<FeedRecommendationEntry>, AppError> {
    let row: Option<(Option<f64>, Option<f64>, Option<i64>, Option<i64>)> = sqlx::query_as(
        "SELECT f.feed, m.milk, dim.days, lac.n
         FROM animals a
         LEFT JOIN LATERAL (SELECT AVG(f.total)::float8 as feed FROM feed_day_amounts f WHERE f.animal_id = a.id AND f.feed_date >= CURRENT_DATE - INTERVAL '7 days') f ON true
         LEFT JOIN LATERAL (SELECT AVG(m.milk_amount)::float8 as milk FROM milk_day_productions m WHERE m.animal_id = a.id AND m.date >= CURRENT_DATE - INTERVAL '30 days') m ON true
         LEFT JOIN LATERAL (SELECT (CURRENT_DATE - c.calving_date)::int8 as days FROM calvings c WHERE c.animal_id = a.id ORDER BY c.calving_date DESC LIMIT 1) dim ON true
         LEFT JOIN LATERAL (SELECT COUNT(*)::int8 as n FROM calvings c WHERE c.animal_id = a.id) lac ON true
         WHERE a.id = $1 AND f.feed IS NOT NULL"
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?;

    let Some((feed_avg, milk_avg, dim_days, lac_count)) = row else { return Ok(None) };
    let current_feed = feed_avg.unwrap_or(0.0);
    let milk = milk_avg.unwrap_or(0.0);
    let dim = dim_days.unwrap_or(150);
    let lac_n = lac_count.unwrap_or(1);

    let base_feed = 12.0 + milk * 0.4;
    let dim_factor = if dim < 60 { 1.15 } else if dim < 120 { 1.05 } else if dim > 250 { 0.9 } else { 1.0 };
    let lac_factor = if lac_n <= 1 { 1.0 } else if lac_n <= 3 { 1.05 } else { 1.1 };
    let recommended = base_feed * dim_factor * lac_factor;
    let diff = recommended - current_feed;
    let suggestion = if diff > 2.0 { "increase_feed".to_string() } else if diff < -2.0 { "reduce_feed".to_string() } else { "maintain".to_string() };
    let name: Option<String> = sqlx::query_scalar("SELECT name FROM animals WHERE id = $1").bind(id).fetch_optional(pool).await.map_err(AppError::Database)?;

    Ok(Some(FeedRecommendationEntry {
        animal_id: id,
        animal_name: name,
        current_feed_avg: round2(current_feed),
        recommended_feed: round2(recommended),
        difference_kg: round2(diff),
        suggestion,
        dim_days: dim as i32,
        lactation_number: lac_n as i32,
        model_version: "rule-based-v1".to_string(),
    }))
}

async fn summary_ketosis(pool: &PgPool, id: i32) -> Result<Option<KetosisWarningEntry>, AppError> {
    let row: Option<(Option<f64>, Option<f64>, Option<i64>, Option<f64>)> = sqlx::query_as(
        "SELECT recent.fpr, fpr_trend.trend, dim.days, rum.rum
         FROM animals a
         LEFT JOIN LATERAL (
             SELECT AVG(q.fat_percentage)::float8 / NULLIF(AVG(q.protein_percentage)::float8, 0) as fpr FROM milk_quality q WHERE q.animal_id = a.id AND q.date >= CURRENT_DATE - INTERVAL '7 days'
         ) recent ON true
         LEFT JOIN LATERAL (
             SELECT (recent.fpr / NULLIF(baseline.fpr, 0) - 1)::float8 as trend FROM
             (SELECT AVG(fat_percentage)::float8 / NULLIF(AVG(protein_percentage)::float8, 0) as fpr FROM milk_quality WHERE animal_id = a.id AND date >= CURRENT_DATE - INTERVAL '7 days') recent,
             (SELECT AVG(fat_percentage)::float8 / NULLIF(AVG(protein_percentage)::float8, 0) as fpr FROM milk_quality WHERE animal_id = a.id AND date >= CURRENT_DATE - INTERVAL '30 days' AND date < CURRENT_DATE - INTERVAL '7 days') baseline
         ) fpr_trend ON true
         LEFT JOIN LATERAL (SELECT (CURRENT_DATE - c.calving_date)::int8 as days FROM calvings c WHERE c.animal_id = a.id ORDER BY c.calving_date DESC LIMIT 1) dim ON true
         LEFT JOIN LATERAL (SELECT AVG(r.rumination_minutes)::float8 as rum FROM ruminations r WHERE r.animal_id = a.id AND r.date >= CURRENT_DATE - INTERVAL '7 days') rum ON true
         WHERE a.id = $1 AND recent.fpr IS NOT NULL"
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?;

    let Some((fpr_opt, fpr_trend, dim_days, rum_7d)) = row else { return Ok(None) };
    let Some(fpr) = fpr_opt else { return Ok(None) };

    let mut risk = 0.0_f64;
    let mut factors = Vec::new();
    let mut risk_type = "subclinical".to_string();
    if fpr < 1.0 { risk += 0.5; factors.push("fpr_below_1.0".to_string()); risk_type = "clinical".to_string(); }
    else if fpr < 1.1 { risk += 0.25; factors.push("fpr_low".to_string()); }
    if let Some(trend) = fpr_trend { if trend < -0.1 { risk += 0.3; factors.push("fpr_declining".to_string()); } }
    if let Some(dim) = dim_days { if dim < 30 { risk += 0.25; factors.push("early_lactation".to_string()); } else if dim < 60 { risk += 0.1; factors.push("fresh_cow".to_string()); } }
    if let Some(rum) = rum_7d { if rum < 400.0 { risk += 0.2; factors.push("low_rumination".to_string()); } }
    if risk < 0.1 { return Ok(None); }
    risk = risk.min(1.0);
    let severity = if risk >= 0.6 { "high" } else if risk >= 0.3 { "moderate" } else { "low" };
    let name: Option<String> = sqlx::query_scalar("SELECT name FROM animals WHERE id = $1").bind(id).fetch_optional(pool).await.map_err(AppError::Database)?;

    Ok(Some(KetosisWarningEntry {
        animal_id: id,
        animal_name: name,
        risk_probability: round2(risk),
        risk_type,
        severity: severity.to_string(),
        fpr_current: round2(fpr),
        fpr_trend: fpr_trend.unwrap_or(0.0),
        contributing_factors: factors,
        model_version: "rule-based-v1".to_string(),
    }))
}

async fn summary_lifetime(pool: &PgPool, id: i32) -> Result<Option<LifetimeValueEntry>, AppError> {
    let row: Option<(Option<f64>, i64, Option<f64>, Option<f64>)> = sqlx::query_as(
        "SELECT EXTRACT(YEAR FROM AGE(CURRENT_DATE, a.birth_date))::float8 as age_years,
                COALESCE(lac_cnt.n, 0) as lac_count,
                total_milk.milk as total_milk,
                avg_milk.avg_m as avg_milk_per_lac
         FROM animals a
         LEFT JOIN LATERAL (SELECT COUNT(*)::int8 as n FROM calvings c WHERE c.animal_id = a.id) lac_cnt ON true
         LEFT JOIN LATERAL (SELECT SUM(m.milk_amount)::float8 as milk FROM milk_day_productions m WHERE m.animal_id = a.id) total_milk ON true
         LEFT JOIN LATERAL (
             SELECT AVG(lac_milk.total)::float8 as avg_m FROM (
                 SELECT SUM(m.milk_amount)::float8 as total FROM milk_day_productions m JOIN calvings c ON c.animal_id = m.animal_id WHERE m.animal_id = a.id AND m.date >= c.calving_date GROUP BY c.id
             ) lac_milk
         ) avg_milk ON true
         WHERE a.id = $1"
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?;

    let Some((age, lac_count, total_milk, avg_milk_per_lac)) = row else { return Ok(None) };
    let max_lacs: i32 = 6;
    let remaining = max_lacs.saturating_sub(lac_count as i32).max(0);
    let avg_per_lac = avg_milk_per_lac.unwrap_or(0.0);
    let proj_milk_val = remaining as f64 * avg_per_lac * 25.0;
    let proj_feed = remaining as f64 * 305.0 * 150.0;
    let net = proj_milk_val - proj_feed;
    let recommendation = if remaining <= 0 || age.unwrap_or(0.0) > 8.0 { "culling_candidate" } else if net < 0.0 { "review" } else if remaining <= 1 { "last_lactation" } else { "keep" };
    let name: Option<String> = sqlx::query_scalar("SELECT name FROM animals WHERE id = $1").bind(id).fetch_optional(pool).await.map_err(AppError::Database)?;

    Ok(Some(LifetimeValueEntry {
        animal_id: id,
        animal_name: name,
        life_number: None,
        age_years: age,
        lactation_count: lac_count,
        total_milk_produced: total_milk,
        estimated_remaining_lactations: remaining,
        projected_milk_value: Some(round2(proj_milk_val)),
        projected_feed_cost: Some(round2(proj_feed)),
        projected_net_value: Some(round2(net)),
        recommendation: recommendation.to_string(),
    }))
}

async fn summary_culling(pool: &PgPool, id: i32) -> Result<Option<CullingSurvivalEntry>, AppError> {
    let row: Option<(Option<i64>, Option<f64>, Option<f64>, Option<f64>, Option<i64>)> = sqlx::query_as(
        "SELECT EXTRACT(YEAR FROM AGE(CURRENT_DATE, a.birth_date))::int8 as age_years,
                latest_milk.milk, avg_scc.scc, ci.interval, lac_count.lacs
         FROM animals a
         LEFT JOIN LATERAL (SELECT AVG(m.milk_amount)::float8 as milk FROM milk_day_productions m WHERE m.animal_id = a.id AND m.date >= CURRENT_DATE - INTERVAL '30 days') latest_milk ON true
         LEFT JOIN LATERAL (SELECT AVG(q.scc)::float8 as scc FROM milk_quality q WHERE q.animal_id = a.id AND q.date >= CURRENT_DATE - INTERVAL '90 days') avg_scc ON true
         LEFT JOIN LATERAL (
             SELECT AVG((c2.calving_date - c1.calving_date))::float8 as interval FROM calvings c1 JOIN calvings c2 ON c1.animal_id = c2.animal_id AND c2.calving_date > c1.calving_date
             WHERE c1.animal_id = a.id AND NOT EXISTS (SELECT 1 FROM calvings c3 WHERE c3.animal_id = c1.animal_id AND c3.calving_date > c1.calving_date AND c3.calving_date < c2.calving_date)
         ) ci ON true
         LEFT JOIN LATERAL (SELECT COUNT(*)::int8 as lacs FROM calvings c WHERE c.animal_id = a.id) lac_count ON true
         WHERE a.id = $1"
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?;

    let Some((age, milk, scc, interval, lac_count)) = row else { return Ok(None) };
    let mut risk = 0.0_f64;
    let mut factors = Vec::new();
    let mut base_days = 730.0_f64;

    if let Some(a) = age { if a >= 10 { risk += 0.4; base_days -= 365.0; factors.push("age>=10yr".to_string()); } else if a >= 8 { risk += 0.25; base_days -= 200.0; factors.push("age>=8yr".to_string()); } else if a >= 6 { risk += 0.1; base_days -= 100.0; factors.push("age>=6yr".to_string()); } }
    if let Some(m) = milk { if m < 15.0 { risk += 0.3; base_days -= 180.0; factors.push("milk<15L".to_string()); } else if m < 20.0 { risk += 0.1; base_days -= 60.0; factors.push("milk<20L".to_string()); } }
    if let Some(s) = scc { if s > 300000.0 { risk += 0.25; base_days -= 180.0; factors.push("SCC>300k".to_string()); } else if s > 200000.0 { risk += 0.1; base_days -= 90.0; factors.push("SCC>200k".to_string()); } }
    if let Some(ci) = interval { if ci > 450.0 { risk += 0.2; base_days -= 120.0; factors.push("interval>450d".to_string()); } else if ci > 400.0 { risk += 0.1; base_days -= 60.0; factors.push("interval>400d".to_string()); } }
    if let Some(lc) = lac_count { if lc >= 6 { risk += 0.1; factors.push("lac>=6".to_string()); } }

    if risk < 0.1 { return Ok(None); }
    risk = risk.min(1.0);
    let expected_days = (base_days * (1.0 - risk)).max(0.0) as i64;
    let name: Option<String> = sqlx::query_scalar("SELECT name FROM animals WHERE id = $1").bind(id).fetch_optional(pool).await.map_err(AppError::Database)?;

    Ok(Some(CullingSurvivalEntry {
        animal_id: id,
        animal_name: name,
        life_number: None,
        expected_days_remaining: Some(expected_days),
        risk_score: round2(risk),
        risk_factors: factors,
    }))
}

async fn summary_cluster(pool: &PgPool, id: i32) -> Result<Option<ClusterCowEntry>, AppError> {
    let row: Option<(Option<f64>, Option<f64>, Option<i64>, Option<i64>)> = sqlx::query_as(
        "SELECT m.avg_milk, r.rum, dim.days, lac.n
         FROM animals a
         LEFT JOIN LATERAL (SELECT AVG(m.milk_amount)::float8 as avg_milk FROM milk_day_productions m WHERE m.animal_id = a.id AND m.date >= CURRENT_DATE - INTERVAL '90 days') m ON true
         LEFT JOIN LATERAL (SELECT AVG(r.rumination_minutes)::float8 as rum FROM ruminations r WHERE r.animal_id = a.id AND r.date >= CURRENT_DATE - INTERVAL '90 days') r ON true
         LEFT JOIN LATERAL (SELECT (CURRENT_DATE - c.calving_date)::int8 as days FROM calvings c WHERE c.animal_id = a.id ORDER BY c.calving_date DESC LIMIT 1) dim ON true
         LEFT JOIN LATERAL (SELECT COUNT(*)::int8 as n FROM calvings c WHERE c.animal_id = a.id) lac ON true
         WHERE a.id = $1 AND m.avg_milk IS NOT NULL"
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?;

    let Some((avg_milk, rum, dim, lac)) = row else { return Ok(None) };
    let milk = avg_milk.unwrap_or(0.0);
    let rumination = rum.unwrap_or(0.0);
    let dim_days = dim.unwrap_or(0) as f64;
    let lac_n = lac.unwrap_or(0) as f64;

    let cluster_id = if milk > 30.0 && rumination > 500.0 { 0 } else if milk > 25.0 { 1 } else if dim_days < 100.0 { 2 } else { 3 };
    let cluster_name = match cluster_id { 0 => "Высокопродуктивные", 1 => "Среднепродуктивные", 2 => "Свежие коровы", _ => "Низкопродуктивные" };
    let name: Option<String> = sqlx::query_scalar("SELECT name FROM animals WHERE id = $1").bind(id).fetch_optional(pool).await.map_err(AppError::Database)?;

    Ok(Some(ClusterCowEntry {
        animal_id: id,
        animal_name: name,
        cluster_id,
        cluster_name: cluster_name.to_string(),
        avg_milk: round2(milk),
        avg_rumination: round2(rumination),
        distance_to_center: round2(((milk - 25.0).powi(2) + (rumination - 450.0).powi(2) + (dim_days - 150.0).powi(2) + (lac_n - 2.0).powi(2)).sqrt()),
        model_version: "rule-based-v1".to_string(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round2() {
        assert_eq!(round2(1.234), 1.23);
        assert_eq!(round2(1.235), 1.24);
        assert_eq!(round2(0.0), 0.0);
        assert_eq!(round2(-1.236), -1.24);
        assert_eq!(round2(99.999), 100.0);
    }

    #[test]
    fn test_wood_predict_basic() {
        let result = wood_predict(20.0, 0.2, 0.003, 60);
        assert!(result > 0.0);
        assert!(result < 100.0);
    }

    #[test]
    fn test_wood_predict_zero_dim() {
        assert_eq!(wood_predict(20.0, 0.2, 0.003, 0), 0.0);
    }

    #[test]
    fn test_wood_predict_negative_dim() {
        assert_eq!(wood_predict(20.0, 0.2, 0.003, -1), 0.0);
    }

    #[test]
    fn test_wood_predict_peak_then_decline() {
        let day30 = wood_predict(20.0, 0.2, 0.003, 30);
        let day60 = wood_predict(20.0, 0.2, 0.003, 60);
        let day200 = wood_predict(20.0, 0.2, 0.003, 200);
        assert!(day60 > day30, "Should still be increasing at day 60");
        assert!(day200 < day60, "Should be declining by day 200");
    }

    #[test]
    fn test_solve_3x3_identity() {
        let a = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let b = [1.0, 2.0, 3.0];
        let result = solve_3x3(a, b).unwrap();
        assert!((result[0] - 1.0).abs() < 1e-10);
        assert!((result[1] - 2.0).abs() < 1e-10);
        assert!((result[2] - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_solve_3x3_singular() {
        let a = [[1.0, 2.0, 3.0], [1.0, 2.0, 3.0], [1.0, 2.0, 3.0]];
        let b = [1.0, 2.0, 3.0];
        assert!(solve_3x3(a, b).is_none());
    }

    #[test]
    fn test_solve_3x3_known_system() {
        let a = [[2.0, 1.0, -1.0], [-3.0, -1.0, 2.0], [-2.0, 1.0, 2.0]];
        let b = [8.0, -11.0, -3.0];
        let result = solve_3x3(a, b).unwrap();
        assert!((result[0] - 2.0).abs() < 1e-10);
        assert!((result[1] - 3.0).abs() < 1e-10);
        assert!((result[2] + 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_fit_wood_model_typical_lactation() {
        let mut data = std::collections::HashMap::new();
        let a = 20.0_f64;
        let b = 0.2_f64;
        let c = 0.003_f64;
        for dim in [10, 20, 30, 40, 50, 60, 80, 100, 150, 200, 250, 300] {
            let milk = a * (dim as f64).powf(b) * (-c * dim as f64).exp();
            data.insert(dim, milk);
        }
        let (fit_a, fit_b, fit_c) = fit_wood_model(&data);
        assert!(fit_a > 0.0, "a should be positive, got {fit_a}");
        assert!(fit_b > 0.0, "b should be positive, got {fit_b}");
        assert!(fit_c > 0.0, "c should be positive, got {fit_c}");

        let orig_60 = wood_predict(a, b, c, 60);
        let fit_60 = wood_predict(fit_a, fit_b, fit_c, 60);
        assert!(
            (fit_60 - orig_60).abs() / orig_60 < 0.05,
            "Fitted value at dim=60 should be within 5%: orig={orig_60}, fit={fit_60}"
        );
    }

    #[test]
    fn test_fit_wood_model_too_few_points() {
        let mut data = std::collections::HashMap::new();
        data.insert(10, 25.0);
        data.insert(20, 28.0);
        let (a, _b, _c) = fit_wood_model(&data);
        assert!(
            a == 25.0 || a == 28.0,
            "Should fall back to one of the data values, got {a}"
        );
    }

    #[test]
    fn test_zscore_normal() {
        let z = zscore(Some(110.0), Some(100.0), Some(10.0));
        assert_eq!(z, Some(1.0));
    }

    #[test]
    fn test_zscore_zero_std() {
        let z = zscore(Some(110.0), Some(100.0), Some(0.0));
        assert_eq!(z, None);
    }

    #[test]
    fn test_zscore_tiny_std() {
        let z = zscore(Some(110.0), Some(100.0), Some(0.005));
        assert_eq!(z, None);
    }

    #[test]
    fn test_zscore_missing_values() {
        assert_eq!(zscore(None, Some(100.0), Some(10.0)), None);
        assert_eq!(zscore(Some(110.0), None, Some(10.0)), None);
        assert_eq!(zscore(Some(110.0), Some(100.0), None), None);
    }
}
