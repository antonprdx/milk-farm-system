use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::config::LelyConfig;
use crate::lely::models::*;

#[derive(Clone)]
pub struct LelyClient {
    http: reqwest::Client,
    base_url: String,
    username: String,
    password: String,
    farm_key: String,
    token: Arc<Mutex<Option<(String, Instant)>>>,
}

impl LelyClient {
    pub fn new(cfg: &LelyConfig) -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            http,
            base_url: cfg.base_url.trim_end_matches('/').to_string(),
            username: cfg.username.clone(),
            password: cfg.password.clone(),
            farm_key: cfg.farm_key.clone(),
            token: Arc::new(Mutex::new(None)),
        }
    }

    async fn authenticate(&self) -> Result<String, anyhow::Error> {
        let url = format!("{}/api/authentication/token", self.base_url);
        let body = AuthenticateRequest {
            user_name: self.username.clone(),
            password: self.password.clone(),
        };

        let resp = self.http.post(&url).json(&body).send().await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Ошибка аутентификации Lely: {} {}", status, text);
        }

        let result: LelyResponse<String> = resp.json().await?;
        let token = result
            .data
            .ok_or_else(|| anyhow::anyhow!("Lely вернул пустой токен"))?;

        tracing::debug!(token_len = token.len(), "Получен токен Lely");
        Ok(token)
    }

    async fn ensure_token(&self) -> Result<String, anyhow::Error> {
        {
            let lock = self.token.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
            if let Some((t, created)) = lock.as_ref()
                && created.elapsed() < Duration::from_secs(3500)
            {
                return Ok(t.clone());
            }
        }

        let new_token = self.authenticate().await?;
        let mut lock = self.token.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
        *lock = Some((new_token.clone(), Instant::now()));
        Ok(new_token)
    }

    async fn get<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        params: &[(&str, &str)],
    ) -> Result<Vec<T>, anyhow::Error> {
        let token = self.ensure_token().await?;
        let url = format!("{}{}", self.base_url, path);

        let max_retries = 2u32;
        let backoff = [1, 3];

        for attempt in 0..=max_retries {
            if attempt > 0 {
                tokio::time::sleep(Duration::from_secs(backoff[attempt as usize - 1])).await;
                tracing::warn!(attempt, path, "Retrying Lely request");
            }

            let mut req = self
                .http
                .get(&url)
                .header("token", &token)
                .header("FarmKey", &self.farm_key);

            for (k, v) in params {
                req = req.query(&[(k, v)]);
            }

            let resp = match req.send().await {
                Ok(r) => r,
                Err(e) if attempt < max_retries && e.is_connect() || e.is_timeout() => {
                    tracing::warn!(error = %e, path, "Lely request failed, will retry");
                    continue;
                }
                Err(e) => return Err(e.into()),
            };

            if !resp.status().is_success() {
                let status = resp.status();
                let text = resp.text().await.unwrap_or_default();
                if attempt < max_retries && status.is_server_error() {
                    tracing::warn!(status = %status, path, "Lely server error, will retry");
                    continue;
                }
                anyhow::bail!("Ошибка запроса Lely {} {}: {}", path, status, text);
            }

            let result: LelyListResponse<T> = resp.json().await?;
            return Ok(result.data.unwrap_or_default());
        }

        unreachable!()
    }

    pub async fn get_animals(&self) -> Result<Vec<AnimalResponse>, anyhow::Error> {
        self.get("/api/animals", &[]).await
    }

    pub async fn get_milk_day_productions(
        &self,
        from: &str,
        till: &str,
    ) -> Result<Vec<DayProduction>, anyhow::Error> {
        self.get(
            "/api/milkdayproductions",
            &[("fromProductionDate", from), ("tillProductionDate", till)],
        )
        .await
    }

    pub async fn get_milk_visits(
        &self,
        from: &str,
        till: &str,
    ) -> Result<Vec<MilkVisit>, anyhow::Error> {
        self.get("/api/milkvisits", &[("fromDate", from), ("tillDate", till)])
            .await
    }

    pub async fn get_milk_visit_quality(
        &self,
        from: &str,
        till: &str,
    ) -> Result<Vec<MilkVisitQuality>, anyhow::Error> {
        self.get(
            "/api/milkvisitsquality",
            &[("fromDate", from), ("tillDate", till)],
        )
        .await
    }

    pub async fn get_milk_day_quality(
        &self,
        from: &str,
        till: &str,
    ) -> Result<Vec<DayProductionQuality>, anyhow::Error> {
        self.get(
            "/api/milkdayproductionsquality",
            &[("fromProductionDate", from), ("tillProductionDate", till)],
        )
        .await
    }

    pub async fn get_robot_data(
        &self,
        from: &str,
        till: &str,
    ) -> Result<Vec<RobotData>, anyhow::Error> {
        self.get(
            "/api/milkvisitrobotdata",
            &[("fromDate", from), ("tillDate", till)],
        )
        .await
    }

    pub async fn get_feed_day_amounts(
        &self,
        from: &str,
        till: &str,
    ) -> Result<Vec<FeedDayAmount>, anyhow::Error> {
        self.get(
            "/api/feedamounts",
            &[("fromFeedDate", from), ("tillFeedDate", till)],
        )
        .await
    }

    pub async fn get_feed_visits(
        &self,
        from: &str,
        till: &str,
    ) -> Result<Vec<FeedVisit>, anyhow::Error> {
        self.get(
            "/api/feedvisits",
            &[("fromFeedDate", from), ("tillFeedDate", till)],
        )
        .await
    }

    pub async fn get_activities(
        &self,
        from: &str,
        till: &str,
    ) -> Result<Vec<Activity>, anyhow::Error> {
        self.get("/api/activities", &[("fromDate", from), ("tillDate", till)])
            .await
    }

    pub async fn get_ruminations(
        &self,
        from: &str,
        till: &str,
    ) -> Result<Vec<Rumination>, anyhow::Error> {
        self.get(
            "/api/ruminations",
            &[("fromActivityDate", from), ("tillActivityDate", till)],
        )
        .await
    }

    pub async fn get_grazing_data(
        &self,
        from: &str,
        till: &str,
    ) -> Result<Vec<Grazing>, anyhow::Error> {
        self.get(
            "/api/grazingdatas",
            &[("fromDate", from), ("tillDate", till)],
        )
        .await
    }

    pub async fn get_calvings(
        &self,
        from: &str,
        till: &str,
    ) -> Result<Vec<Calving>, anyhow::Error> {
        self.get("/api/calvings", &[("fromDate", from), ("tillDate", till)])
            .await
    }

    pub async fn get_inseminations(
        &self,
        from: &str,
        till: &str,
    ) -> Result<Vec<Insemination>, anyhow::Error> {
        self.get(
            "/api/inseminations",
            &[
                ("fromInseminationDate", from),
                ("tillInseminationDate", till),
            ],
        )
        .await
    }

    pub async fn get_pregnancies(
        &self,
        from: &str,
        till: &str,
    ) -> Result<Vec<Pregnancy>, anyhow::Error> {
        self.get(
            "/api/pregnancies",
            &[("fromDate", from), ("tillDate", till)],
        )
        .await
    }

    pub async fn get_heats(&self, from: &str, till: &str) -> Result<Vec<Heat>, anyhow::Error> {
        self.get("/api/heats", &[("fromDate", from), ("tillDate", till)])
            .await
    }

    pub async fn get_dry_offs(&self, from: &str, till: &str) -> Result<Vec<DryOff>, anyhow::Error> {
        self.get("/api/dryoffs", &[("fromDate", from), ("tillDate", till)])
            .await
    }

    pub async fn get_sires(&self) -> Result<Vec<Sire>, anyhow::Error> {
        self.get("/api/sires", &[]).await
    }

    pub async fn get_transfers(
        &self,
        from: &str,
        till: &str,
    ) -> Result<Vec<Transfer>, anyhow::Error> {
        self.get(
            "/api/transfers",
            &[("fromTransferDate", from), ("tillTransferDate", till)],
        )
        .await
    }

    pub async fn get_bloodlines(&self, life_number: &str) -> Result<Vec<BloodLine>, anyhow::Error> {
        self.get("/api/bloodlines", &[("lifeNumber", life_number)])
            .await
    }

    pub async fn get_feed_types(&self) -> Result<Vec<FeedType>, anyhow::Error> {
        self.get("/api/feedtypes", &[]).await
    }

    pub async fn get_feed_groups(&self) -> Result<Vec<FeedGroup>, anyhow::Error> {
        self.get("/api/feedgroups", &[]).await
    }

    pub async fn get_contacts(&self) -> Result<Vec<Contact>, anyhow::Error> {
        self.get("/api/contacts", &[]).await
    }

    pub async fn get_locations(&self) -> Result<Vec<Location>, anyhow::Error> {
        self.get("/api/locations", &[]).await
    }
}
