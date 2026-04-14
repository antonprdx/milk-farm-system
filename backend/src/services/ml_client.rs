use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::errors::AppError;
use crate::models::analytics::{
    CullingSurvivalEntry, CullingSurvivalResponse, MastitisRiskEntry, MastitisRiskResponse,
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
