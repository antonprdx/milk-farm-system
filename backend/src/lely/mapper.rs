use std::collections::HashMap;

use sqlx::PgPool;

#[derive(Clone)]
pub struct AnimalCache {
    pub by_life: HashMap<String, i32>,
}

impl AnimalCache {
    pub async fn load(pool: &PgPool) -> Result<Self, sqlx::Error> {
        let rows: Vec<(i32, Option<String>)> =
            sqlx::query_as("SELECT id, life_number FROM animals WHERE life_number IS NOT NULL")
                .fetch_all(pool)
                .await?;

        let by_life = rows
            .into_iter()
            .filter_map(|(id, ln)| ln.map(|l| (l, id)))
            .collect();

        Ok(Self { by_life })
    }

    pub fn resolve(&self, life_number: &Option<String>) -> Option<i32> {
        life_number
            .as_ref()
            .and_then(|ln| self.by_life.get(ln))
            .copied()
    }

    pub fn resolve_or_warn(&self, life_number: &Option<String>, entity: &str) -> Option<i32> {
        let Some(ln) = life_number.as_ref() else {
            tracing::debug!(entity, "Запись без life_number, пропущена");
            return None;
        };
        match self.by_life.get(ln) {
            Some(id) => Some(*id),
            None => {
                tracing::warn!(entity, life_number = %ln, "Животное не найдено в БД, запись пропущена");
                None
            }
        }
    }
}
