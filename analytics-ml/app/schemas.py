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


class TrainRequest(BaseModel):
    model_name: str


class TrainResponse(BaseModel):
    model_name: str
    samples: int
    metrics: dict[str, float]
    duration_seconds: float
