use clickhouse::{Client, Row};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Row, Serialize, Deserialize, Debug)]
struct MilkRow {
    date: String,
    animal_id: u32,
    milk_amount: f64,
}

#[derive(Row, Serialize, Deserialize, Debug)]
struct MilkQualityRow {
    date: String,
    animal_id: u32,
    scc: f64,
    fat_percentage: f64,
    protein_percentage: f64,
    lactose_percentage: f64,
}

#[derive(Row, Serialize, Deserialize, Debug)]
struct RuminationRow {
    date: String,
    animal_id: u32,
    rumination_minutes: f64,
}

#[derive(Row, Serialize, Deserialize, Debug)]
struct FeedRow {
    date: String,
    animal_id: u32,
    total: f64,
}

#[derive(Row, Serialize, Deserialize, Debug)]
struct ActivityRow {
    date: String,
    animal_id: u32,
    activity_counter: f64,
}

pub struct ClickhouseSync {
    client: Client,
    batch_size: usize,
}

impl ClickhouseSync {
    pub fn new(url: &str) -> Self {
        let client = Client::default()
            .with_url(url)
            .with_database("milkfarm_analytics");
        Self {
            client,
            batch_size: 50000,
        }
    }

    pub async fn health_check(&self) -> bool {
        self.client
            .query("SELECT 1")
            .execute()
            .await
            .is_ok()
    }

    pub async fn sync_milk(&self, pool: &PgPool) -> Result<u64, String> {
        let rows: Vec<(String, i32, f64)> = sqlx::query_as(
            "SELECT date::text, animal_id, milk_amount
             FROM milk_day_productions
             WHERE date >= get_ref_date() - INTERVAL '60 days'"
        )
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

        let count = rows.len();
        for chunk in rows.chunks(self.batch_size) {
            let mut insert = self.client.insert::<MilkRow>("milk_day_productions").map_err(|e| e.to_string())?;
            for (d, aid, m) in chunk {
                insert.write(&MilkRow { date: d.clone(), animal_id: *aid as u32, milk_amount: *m }).await.map_err(|e| e.to_string())?;
            }
            insert.end().await.map_err(|e| e.to_string())?;
        }
        tracing::info!("Synced {} milk rows to ClickHouse", count);
        Ok(count as u64)
    }

    pub async fn sync_milk_quality(&self, pool: &PgPool) -> Result<u64, String> {
        let rows: Vec<(String, i32, f64, f64, f64, f64)> = sqlx::query_as(
            "SELECT date::text, animal_id,
                    COALESCE(scc, 0), COALESCE(fat_percentage, 0),
                    COALESCE(protein_percentage, 0), COALESCE(lactose_percentage, 0)
             FROM milk_quality
             WHERE date >= get_ref_date() - INTERVAL '60 days'"
        )
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

        let count = rows.len();
        for chunk in rows.chunks(self.batch_size) {
            let mut insert = self.client.insert::<MilkQualityRow>("milk_quality").map_err(|e| e.to_string())?;
            for (d, aid, scc, fat, prot, lac) in chunk {
                insert.write(&MilkQualityRow {
                    date: d.clone(), animal_id: *aid as u32,
                    scc: *scc, fat_percentage: *fat, protein_percentage: *prot, lactose_percentage: *lac,
                }).await.map_err(|e| e.to_string())?;
            }
            insert.end().await.map_err(|e| e.to_string())?;
        }
        tracing::info!("Synced {} milk_quality rows to ClickHouse", count);
        Ok(count as u64)
    }

    pub async fn sync_ruminations(&self, pool: &PgPool) -> Result<u64, String> {
        let rows: Vec<(String, i32, f64)> = sqlx::query_as(
            "SELECT date::text, animal_id, rumination_minutes
             FROM ruminations WHERE date >= get_ref_date() - INTERVAL '60 days'"
        )
        .fetch_all(pool).await.map_err(|e| e.to_string())?;

        let count = rows.len();
        for chunk in rows.chunks(self.batch_size) {
            let mut insert = self.client.insert::<RuminationRow>("ruminations").map_err(|e| e.to_string())?;
            for (d, aid, r) in chunk {
                insert.write(&RuminationRow { date: d.clone(), animal_id: *aid as u32, rumination_minutes: *r }).await.map_err(|e| e.to_string())?;
            }
            insert.end().await.map_err(|e| e.to_string())?;
        }
        tracing::info!("Synced {} rumination rows to ClickHouse", count);
        Ok(count as u64)
    }

    pub async fn sync_feed(&self, pool: &PgPool) -> Result<u64, String> {
        let rows: Vec<(String, i32, f64)> = sqlx::query_as(
            "SELECT feed_date::text, animal_id, total
             FROM feed_day_amounts WHERE feed_date >= get_ref_date() - INTERVAL '60 days'"
        )
        .fetch_all(pool).await.map_err(|e| e.to_string())?;

        let count = rows.len();
        for chunk in rows.chunks(self.batch_size) {
            let mut insert = self.client.insert::<FeedRow>("feed_day_amounts").map_err(|e| e.to_string())?;
            for (d, aid, t) in chunk {
                insert.write(&FeedRow { date: d.clone(), animal_id: *aid as u32, total: *t }).await.map_err(|e| e.to_string())?;
            }
            insert.end().await.map_err(|e| e.to_string())?;
        }
        tracing::info!("Synced {} feed rows to ClickHouse", count);
        Ok(count as u64)
    }

    pub async fn sync_activities(&self, pool: &PgPool) -> Result<u64, String> {
        let rows: Vec<(String, i32, f64)> = sqlx::query_as(
            "SELECT date(activity_datetime)::text, animal_id, activity_counter
             FROM activities WHERE activity_datetime >= get_ref_date() - INTERVAL '60 days'"
        )
        .fetch_all(pool).await.map_err(|e| e.to_string())?;

        let count = rows.len();
        for chunk in rows.chunks(self.batch_size) {
            let mut insert = self.client.insert::<ActivityRow>("activities").map_err(|e| e.to_string())?;
            for (d, aid, a) in chunk {
                insert.write(&ActivityRow { date: d.clone(), animal_id: *aid as u32, activity_counter: *a }).await.map_err(|e| e.to_string())?;
            }
            insert.end().await.map_err(|e| e.to_string())?;
        }
        tracing::info!("Synced {} activity rows to ClickHouse", count);
        Ok(count as u64)
    }

    pub async fn sync_all(&self, pool: &PgPool) -> Result<(), String> {
        let _ = self.sync_milk(pool).await;
        let _ = self.sync_milk_quality(pool).await;
        let _ = self.sync_ruminations(pool).await;
        let _ = self.sync_feed(pool).await;
        let _ = self.sync_activities(pool).await;
        Ok(())
    }
}
