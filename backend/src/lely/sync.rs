use std::sync::Arc;
use std::time::Duration;

use chrono::Datelike;
use futures::future::BoxFuture;

use crate::lely::client::LelyClient;
use crate::lely::mapper::AnimalCache;
use crate::lely::service;
use crate::state::AppStateInner;

pub fn start_sync_scheduler(state: Arc<AppStateInner>) {
    let cfg = state.lely.get_config();
    let interval = Duration::from_secs(cfg.sync_interval_secs);
    let cancel = state.lely.cancel.read().unwrap().clone();

    tokio::spawn(async move {
        tracing::info!(
            interval_secs = interval.as_secs(),
            "Планировщик синхронизации Lely запущен"
        );

        loop {
            tokio::select! {
                _ = tokio::time::sleep(interval) => {}
                _ = cancel.cancelled() => {
                    tracing::info!("Планировщик синхронизации Lely остановлен");
                    return;
                }
            }

            match run_sync(&state).await {
                Ok(()) => {
                    tracing::info!("Цикл синхронизации Lely завершён");
                }
                Err(e) => {
                    tracing::error!(error = %e, "Ошибка цикла синхронизации Lely");
                }
            }
        }
    });
}

pub async fn run_sync(state: &Arc<AppStateInner>) -> Result<(), anyhow::Error> {
    if !state.lely.get_config().enabled {
        return Ok(());
    }

    let locked = service::try_acquire_lock(&state.pool).await?;
    if !locked {
        tracing::warn!("Синхронизация Lely уже выполняется, пропуск");
        return Ok(());
    }

    let result = run_sync_inner(state).await;

    if let Err(e) = service::release_lock(&state.pool).await {
        tracing::warn!(error = %e, "Не удалось освободить блокировку Lely");
    }

    result
}

macro_rules! try_sync {
    ($expr:expr) => {
        if let Err(e) = $expr.await {
            tracing::error!(error = %e, "Ошибка синхронизации (продолжаем)");
        }
    };
}

async fn run_sync_inner(state: &Arc<AppStateInner>) -> Result<(), anyhow::Error> {
    let cfg = state.lely.get_config();
    let client = LelyClient::new(&cfg);
    let pool = &state.pool;

    try_sync!(sync_simple(
        pool,
        "animals",
        Box::pin(async {
            let records = client.get_animals().await?;
            service::upsert_animals(pool, &records).await
        }),
    ));

    let cache = AnimalCache::load(pool).await?;

    try_sync!(sync_chunked(
        pool,
        "milk_day_productions",
        10,
        |from, till| {
            let c = client.clone();
            let p = pool.clone();
            let cache = cache.clone();
            Box::pin(async move {
                let records = c.get_milk_day_productions(&from, &till).await?;
                service::upsert_milk_day_productions(&p, &records, &cache).await
            })
        }
    ));

    try_sync!(sync_chunked(pool, "milk_visits", 7, |from, till| {
        let c = client.clone();
        let p = pool.clone();
        let cache = cache.clone();
        Box::pin(async move {
            let records = c.get_milk_visits(&from, &till).await?;
            service::upsert_milk_visits(&p, &records, &cache).await
        })
    }));

    try_sync!(sync_chunked(pool, "milk_visit_quality", 1, |from, till| {
        let c = client.clone();
        let p = pool.clone();
        let cache = cache.clone();
        Box::pin(async move {
            let records = c.get_milk_visit_quality(&from, &till).await?;
            service::upsert_milk_visit_quality(&p, &records, &cache).await
        })
    }));

    try_sync!(sync_chunked(
        pool,
        "milk_day_productions_quality",
        1,
        |from, till| {
            let c = client.clone();
            let p = pool.clone();
            let cache = cache.clone();
            Box::pin(async move {
                let records = c.get_milk_day_quality(&from, &till).await?;
                service::upsert_milk_day_quality(&p, &records, &cache).await
            })
        }
    ));

    try_sync!(sync_chunked(pool, "robot_milk_data", 7, |from, till| {
        let c = client.clone();
        let p = pool.clone();
        let cache = cache.clone();
        Box::pin(async move {
            let records = c.get_robot_data(&from, &till).await?;
            service::upsert_robot_data(&p, &records, &cache).await
        })
    }));

    try_sync!(sync_chunked(pool, "feed_day_amounts", 7, |from, till| {
        let c = client.clone();
        let p = pool.clone();
        let cache = cache.clone();
        Box::pin(async move {
            let records = c.get_feed_day_amounts(&from, &till).await?;
            service::upsert_feed_day_amounts(&p, &records, &cache).await
        })
    }));

    try_sync!(sync_chunked(pool, "feed_visits", 3, |from, till| {
        let c = client.clone();
        let p = pool.clone();
        let cache = cache.clone();
        Box::pin(async move {
            let records = c.get_feed_visits(&from, &till).await?;
            service::upsert_feed_visits(&p, &records, &cache).await
        })
    }));

    try_sync!(sync_chunked(pool, "activities", 1, |from, till| {
        let c = client.clone();
        let p = pool.clone();
        let cache = cache.clone();
        Box::pin(async move {
            let records = c.get_activities(&from, &till).await?;
            service::upsert_activities(&p, &records, &cache).await
        })
    }));

    try_sync!(sync_chunked(pool, "ruminations", 2, |from, till| {
        let c = client.clone();
        let p = pool.clone();
        let cache = cache.clone();
        Box::pin(async move {
            let records = c.get_ruminations(&from, &till).await?;
            service::upsert_ruminations(&p, &records, &cache).await
        })
    }));

    try_sync!(sync_grazing(&client, pool));

    try_sync!(sync_chunked(pool, "calvings", 90, |from, till| {
        let c = client.clone();
        let p = pool.clone();
        let cache = cache.clone();
        Box::pin(async move {
            let records = c.get_calvings(&from, &till).await?;
            service::upsert_calvings(&p, &records, &cache).await
        })
    }));

    try_sync!(sync_chunked(pool, "inseminations", 90, |from, till| {
        let c = client.clone();
        let p = pool.clone();
        let cache = cache.clone();
        Box::pin(async move {
            let records = c.get_inseminations(&from, &till).await?;
            service::upsert_inseminations(&p, &records, &cache).await
        })
    }));

    try_sync!(sync_chunked(pool, "pregnancies", 90, |from, till| {
        let c = client.clone();
        let p = pool.clone();
        let cache = cache.clone();
        Box::pin(async move {
            let records = c.get_pregnancies(&from, &till).await?;
            service::upsert_pregnancies(&p, &records, &cache).await
        })
    }));

    try_sync!(sync_chunked(pool, "heats", 90, |from, till| {
        let c = client.clone();
        let p = pool.clone();
        let cache = cache.clone();
        Box::pin(async move {
            let records = c.get_heats(&from, &till).await?;
            service::upsert_heats(&p, &records, &cache).await
        })
    }));

    try_sync!(sync_chunked(pool, "dry_offs", 90, |from, till| {
        let c = client.clone();
        let p = pool.clone();
        let cache = cache.clone();
        Box::pin(async move {
            let records = c.get_dry_offs(&from, &till).await?;
            service::upsert_dry_offs(&p, &records, &cache).await
        })
    }));

    try_sync!(sync_simple(
        pool,
        "sires",
        Box::pin(async {
            let records = client.get_sires().await?;
            service::upsert_sires(pool, &records).await
        }),
    ));

    try_sync!(sync_chunked(pool, "transfers", 90, |from, till| {
        let c = client.clone();
        let p = pool.clone();
        let cache = cache.clone();
        Box::pin(async move {
            let records = c.get_transfers(&from, &till).await?;
            service::upsert_transfers(&p, &records, &cache).await
        })
    }));

    try_sync!(sync_bloodlines(&client, pool, &cache));

    for entity in &["feed_types", "feed_groups", "contacts", "locations"] {
        tracing::info!(entity, "Синхронизация (stub) начата");
        service::update_sync_state(pool, entity, "success", 0, None).await?;
        tracing::info!(entity, "Синхронизация (stub) завершена");
    }

    Ok(())
}

async fn sync_simple(
    pool: &sqlx::PgPool,
    entity: &str,
    fut: BoxFuture<'_, Result<u64, anyhow::Error>>,
) -> Result<(), anyhow::Error> {
    tracing::info!(entity, "Синхронизация начата");
    match fut.await {
        Ok(count) => {
            service::update_sync_state(pool, entity, "success", count as i64, None as Option<&str>)
                .await?;
            tracing::info!(entity, count, "Синхронизация завершена");
            Ok(())
        }
        Err(e) => {
            tracing::error!(entity, error = %e, "Ошибка синхронизации");
            service::update_sync_state(pool, entity, "error", 0i64, Some(&e.to_string())).await?;
            Err(e)
        }
    }
}

async fn sync_chunked(
    pool: &sqlx::PgPool,
    entity: &str,
    max_days: i64,
    sync_fn: impl Fn(String, String) -> BoxFuture<'static, Result<u64, anyhow::Error>>,
) -> Result<(), anyhow::Error> {
    tracing::info!(entity, "Синхронизация начата");
    let state = service::get_sync_state(pool, entity).await?;

    let since = state
        .and_then(|s| s.last_synced_at)
        .map(|dt| dt.date_naive() - chrono::Duration::days(1))
        .unwrap_or_else(|| chrono::Utc::now().date_naive() - chrono::Duration::days(30));

    let now = chrono::Utc::now().date_naive();
    let mut total: u64 = 0;
    let mut current = since;

    while current < now {
        let chunk_end = (current + chrono::Duration::days(max_days)).min(now);
        let from = current.format("%Y-%m-%dT00:00:00Z").to_string();
        let till = chunk_end.format("%Y-%m-%dT23:59:59Z").to_string();

        match sync_fn(from, till).await {
            Ok(count) => {
                total += count;
            }
            Err(e) => {
                tracing::error!(entity, error = %e, "Ошибка чанковой синхронизации");
                service::update_sync_state(
                    pool,
                    entity,
                    "error",
                    total as i64,
                    Some(&e.to_string()),
                )
                .await?;
                return Err(e);
            }
        }

        current = chunk_end;
    }

    service::update_sync_state(pool, entity, "success", total as i64, None).await?;
    tracing::info!(entity, total, "Синхронизация завершена");
    Ok(())
}

async fn sync_grazing(client: &LelyClient, pool: &sqlx::PgPool) -> Result<(), anyhow::Error> {
    tracing::info!("Синхронизация: grazing_data");
    let state = service::get_sync_state(pool, "grazing_data").await?;

    let since = state
        .and_then(|s| s.last_synced_at)
        .map(|dt| dt.date_naive() - chrono::Duration::days(1))
        .unwrap_or_else(|| {
            chrono::NaiveDate::from_ymd_opt(chrono::Utc::now().year(), 1, 1)
                .unwrap_or(chrono::Utc::now().date_naive())
        });

    let now = chrono::Utc::now().date_naive();
    let mut total: u64 = 0;
    let mut current = since;

    while current < now {
        let chunk_end = (current + chrono::Duration::days(90)).min(now);
        let from = current.format("%Y-%m-%dT00:00:00Z").to_string();
        let till = chunk_end.format("%Y-%m-%dT23:59:59Z").to_string();

        match client.get_grazing_data(&from, &till).await {
            Ok(records) => {
                let count = service::upsert_grazing_data(pool, &records).await?;
                total += count;
            }
            Err(e) => {
                service::update_sync_state(
                    pool,
                    "grazing_data",
                    "error",
                    total as i64,
                    Some(&e.to_string()),
                )
                .await?;
                return Err(e);
            }
        }

        current = chunk_end;
    }

    service::update_sync_state(pool, "grazing_data", "success", total as i64, None).await?;
    tracing::info!(total, "Синхронизация: grazing_data завершена");
    Ok(())
}

async fn sync_bloodlines(
    client: &LelyClient,
    pool: &sqlx::PgPool,
    cache: &AnimalCache,
) -> Result<(), anyhow::Error> {
    tracing::info!("Синхронизация: bloodlines");
    let mut total: u64 = 0;

    for ln in cache.by_life.keys() {
        match client.get_bloodlines(ln).await {
            Ok(records) => {
                let count = service::upsert_bloodlines(pool, &records, cache).await?;
                total += count;
            }
            Err(e) => {
                tracing::warn!(life_number = %ln, error = %e, "Ошибка синхронизации bloodlines");
            }
        }
    }

    service::update_sync_state(pool, "bloodlines", "success", total as i64, None).await?;
    tracing::info!(total, "Синхронизация: bloodlines завершена");
    Ok(())
}
