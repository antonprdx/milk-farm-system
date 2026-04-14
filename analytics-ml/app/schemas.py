from pydantic import BaseModel


class MastitisRiskRequest(BaseModel):
    animal_id: int | None = None


class MastitisRiskPrediction(BaseModel):
    animal_id: int
    animal_name: str | None
    risk_probability: float
    risk_level: str
    contributing_features: list[str]
    model_version: str


class MastitisRiskResponse(BaseModel):
    predictions: list[MastitisRiskPrediction]


class CullingRiskRequest(BaseModel):
    animal_id: int | None = None


class CullingRiskPrediction(BaseModel):
    animal_id: int
    animal_name: str | None
    risk_probability: float
    expected_days_remaining: int | None
    risk_factors: list[str]
    model_version: str


class CullingRiskResponse(BaseModel):
    predictions: list[CullingRiskPrediction]


class MilkForecastRequest(BaseModel):
    animal_id: int
    days: int = 30


class MilkForecastDay(BaseModel):
    day_offset: int
    predicted_milk: float
    lower_bound: float
    upper_bound: float


class MilkForecastResponse(BaseModel):
    animal_id: int
    animal_name: str | None
    current_daily_avg: float | None
    forecast: list[MilkForecastDay]
    model_version: str


class HealthReport(BaseModel):
    status: str
    model_dir: str
    models: dict[str, str | None]
    database_connected: bool


class ClusterRequest(BaseModel):
    days: int = 90


class ClusterEntry(BaseModel):
    animal_id: int
    animal_name: str | None
    cluster_id: int
    cluster_name: str
    avg_milk: float
    avg_rumination: float
    distance_to_center: float
    model_version: str


class ClusterResponse(BaseModel):
    clusters: list[ClusterEntry]
    cluster_names: dict[str, str]


class TrainRequest(BaseModel):
    model_name: str


class TrainResponse(BaseModel):
    model_name: str
    samples: int
    metrics: dict[str, float | dict[str, int]]
    duration_seconds: float


class EstrusRequest(BaseModel):
    animal_id: int | None = None


class EstrusPrediction(BaseModel):
    animal_id: int
    animal_name: str | None
    estrus_probability: float
    status: str
    contributing_signals: list[str]
    optimal_window: str | None
    model_version: str


class EstrusResponse(BaseModel):
    predictions: list[EstrusPrediction]


class EquipmentAnomalyRequest(BaseModel):
    pass


class EquipmentAnomalyEntry(BaseModel):
    animal_id: int
    animal_name: str | None
    is_anomaly: bool
    anomaly_score: float
    severity: str
    flags: list[str]
    device_address: int | None
    model_version: str


class EquipmentAnomalyResponse(BaseModel):
    entries: list[EquipmentAnomalyEntry]


class FeedRecommendationRequest(BaseModel):
    animal_id: int | None = None


class FeedRecommendationEntry(BaseModel):
    animal_id: int
    animal_name: str | None
    current_feed_avg: float
    recommended_feed: float
    difference_kg: float
    suggestion: str
    dim_days: int
    lactation_number: int
    model_version: str


class FeedRecommendationResponse(BaseModel):
    recommendations: list[FeedRecommendationEntry]


class KetosisWarningRequest(BaseModel):
    animal_id: int | None = None


class KetosisWarningEntry(BaseModel):
    animal_id: int
    animal_name: str | None
    risk_probability: float
    risk_type: str
    severity: str
    fpr_current: float
    fpr_trend: float
    contributing_factors: list[str]
    model_version: str


class KetosisWarningResponse(BaseModel):
    predictions: list[KetosisWarningEntry]
