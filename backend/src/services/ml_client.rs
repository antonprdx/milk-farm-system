use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::errors::AppError;
use crate::models::analytics::{
    ClusterCowEntry, CowClusterResponse, CullingSurvivalEntry, CullingSurvivalResponse,
    MastitisRiskEntry, MastitisRiskResponse, MilkForecastDataResponse, MilkForecastDay,
};

#[derive(Debug, Clone)]
pub struct MlClient {
    base_url: String,
    client: reqwest::Client,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct MlHealthResponse {
    status: String,
    models: std::collections::HashMap<String, Option<String>>,
}

#[derive(Debug, Serialize)]
struct MastitisRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    animal_id: Option<i32>,
}

#[derive(Debug, Serialize)]
struct CullingRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    animal_id: Option<i32>,
}

#[derive(Debug, Serialize)]
struct ForecastRequest {
    animal_id: i32,
    days: i32,
}

#[derive(Debug, Serialize)]
struct ClusterRequest {
    days: i32,
}

#[derive(Debug, Deserialize)]
struct MlMastitisPrediction {
    animal_id: i32,
    animal_name: Option<String>,
    risk_probability: f64,
    risk_level: String,
    contributing_features: Vec<String>,
    #[allow(dead_code)]
    model_version: String,
}

#[derive(Debug, Deserialize)]
struct MlMastitisResponse {
    predictions: Vec<MlMastitisPrediction>,
}

#[derive(Debug, Deserialize)]
struct MlCullingPrediction {
    animal_id: i32,
    animal_name: Option<String>,
    risk_probability: f64,
    expected_days_remaining: Option<i64>,
    risk_factors: Vec<String>,
    #[allow(dead_code)]
    model_version: String,
}

#[derive(Debug, Deserialize)]
struct MlCullingResponse {
    predictions: Vec<MlCullingPrediction>,
}

#[derive(Debug, Deserialize)]
struct MlForecastResponse {
    #[allow(dead_code)]
    animal_id: i32,
    animal_name: Option<String>,
    current_daily_avg: Option<f64>,
    forecast: Vec<MlForecastDay>,
    model_version: String,
}

#[derive(Debug, Deserialize)]
struct MlForecastDay {
    day_offset: i32,
    predicted_milk: f64,
    lower_bound: f64,
    upper_bound: f64,
}

#[derive(Debug, Deserialize)]
struct MlClusterEntry {
    animal_id: i32,
    animal_name: Option<String>,
    cluster_id: i32,
    cluster_name: String,
    avg_milk: f64,
    avg_rumination: f64,
    distance_to_center: f64,
    model_version: String,
}

#[derive(Debug, Deserialize)]
struct MlClusterResponse {
    clusters: Vec<MlClusterEntry>,
    cluster_names: std::collections::HashMap<String, String>,
}

impl MlClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
        }
    }

    pub fn from_env() -> Option<Self> {
        let url = std::env::var("ML_SERVICE_URL").unwrap_or_default();
        if url.is_empty() {
            return None;
        }
        Some(Self::new(url))
    }

    async fn is_healthy(&self) -> bool {
        match self
            .client
            .get(format!("{}/health", self.base_url))
            .send()
            .await
        {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }

    pub async fn mastitis_risk(
        &self,
        animal_id: Option<i32>,
        fallback_pool: &PgPool,
    ) -> Result<MastitisRiskResponse, AppError> {
        if self.is_healthy().await {
            match self.try_ml_mastitis(animal_id).await {
                Ok(resp) => return Ok(resp),
                Err(e) => {
                    tracing::warn!("ML mastitis failed, falling back to rule-based: {}", e);
                }
            }
        }
        super::predictive_service::mastitis_risk(fallback_pool).await
    }

    pub async fn culling_survival(
        &self,
        animal_id: Option<i32>,
        fallback_pool: &PgPool,
    ) -> Result<CullingSurvivalResponse, AppError> {
        if self.is_healthy().await {
            match self.try_ml_culling(animal_id).await {
                Ok(resp) => return Ok(resp),
                Err(e) => {
                    tracing::warn!("ML culling failed, falling back to rule-based: {}", e);
                }
            }
        }
        super::predictive_service::culling_survival(fallback_pool).await
    }

    pub async fn milk_forecast(
        &self,
        animal_id: i32,
        days: i32,
    ) -> Result<MilkForecastDataResponse, AppError> {
        let resp = self
            .client
            .post(format!("{}/predict/milk-forecast", self.base_url))
            .json(&ForecastRequest { animal_id, days })
            .send()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!(e.to_string())))?;

        if !resp.status().is_success() {
            return Err(AppError::Internal(anyhow::anyhow!(
                "ML service returned {}",
                resp.status()
            )));
        }

        let ml_resp: MlForecastResponse = resp
            .json()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!(e.to_string())))?;

        Ok(MilkForecastDataResponse {
            animal_id: ml_resp.animal_id,
            animal_name: ml_resp.animal_name,
            current_daily_avg: ml_resp.current_daily_avg,
            forecast: ml_resp
                .forecast
                .into_iter()
                .map(|d| MilkForecastDay {
                    day_offset: d.day_offset,
                    predicted_milk: d.predicted_milk,
                    lower_bound: d.lower_bound,
                    upper_bound: d.upper_bound,
                })
                .collect(),
            model_version: ml_resp.model_version,
        })
    }

    pub async fn cow_clusters(
        &self,
        days: i32,
    ) -> Result<CowClusterResponse, AppError> {
        let resp = self
            .client
            .post(format!("{}/predict/clusters", self.base_url))
            .json(&ClusterRequest { days })
            .send()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!(e.to_string())))?;

        if !resp.status().is_success() {
            return Err(AppError::Internal(anyhow::anyhow!(
                "ML service returned {}",
                resp.status()
            )));
        }

        let ml_resp: MlClusterResponse = resp
            .json()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!(e.to_string())))?;

        let clusters: Vec<ClusterCowEntry> = ml_resp
            .clusters
            .into_iter()
            .map(|c| ClusterCowEntry {
                animal_id: c.animal_id,
                animal_name: c.animal_name,
                cluster_id: c.cluster_id,
                cluster_name: c.cluster_name,
                avg_milk: c.avg_milk,
                avg_rumination: c.avg_rumination,
                distance_to_center: c.distance_to_center,
                model_version: c.model_version,
            })
            .collect();

        Ok(CowClusterResponse {
            cluster_names: ml_resp.cluster_names,
            clusters,
        })
    }

    async fn try_ml_mastitis(
        &self,
        animal_id: Option<i32>,
    ) -> Result<MastitisRiskResponse, AppError> {
        let resp = self
            .client
            .post(format!("{}/predict/mastitis", self.base_url))
            .json(&MastitisRequest { animal_id })
            .send()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!(e.to_string())))?;

        if !resp.status().is_success() {
            return Err(AppError::Internal(anyhow::anyhow!(
                "ML service returned {}",
                resp.status()
            )));
        }

        let ml_resp: MlMastitisResponse = resp
            .json()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!(e.to_string())))?;

        let cows: Vec<MastitisRiskEntry> = ml_resp
            .predictions
            .into_iter()
            .map(|p| MastitisRiskEntry {
                animal_id: p.animal_id,
                animal_name: p.animal_name,
                life_number: None,
                risk_score: p.risk_probability,
                risk_level: p.risk_level,
                contributing_factors: p.contributing_features,
            })
            .collect();

        Ok(MastitisRiskResponse {
            cows,
            model_version: "xgboost-v1".to_string(),
        })
    }

    async fn try_ml_culling(
        &self,
        animal_id: Option<i32>,
    ) -> Result<CullingSurvivalResponse, AppError> {
        let resp = self
            .client
            .post(format!("{}/predict/culling", self.base_url))
            .json(&CullingRequest { animal_id })
            .send()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!(e.to_string())))?;

        if !resp.status().is_success() {
            return Err(AppError::Internal(anyhow::anyhow!(
                "ML service returned {}",
                resp.status()
            )));
        }

        let ml_resp: MlCullingResponse = resp
            .json()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!(e.to_string())))?;

        let cows: Vec<CullingSurvivalEntry> = ml_resp
            .predictions
            .into_iter()
            .map(|p| CullingSurvivalEntry {
                animal_id: p.animal_id,
                animal_name: p.animal_name,
                life_number: None,
                expected_days_remaining: p.expected_days_remaining,
                risk_score: p.risk_probability,
                risk_factors: p.risk_factors,
            })
            .collect();

        Ok(CullingSurvivalResponse {
            cows,
            model_version: "xgboost-v1".to_string(),
        })
    }
}
