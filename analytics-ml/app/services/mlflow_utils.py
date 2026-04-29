import mlflow
from app.config import settings


def get_experiment_id(name: str) -> str | None:
    mlflow.set_tracking_uri(settings.mlflow_tracking_uri)
    exp = mlflow.get_experiment_by_name(name)
    if exp is None:
        exp = mlflow.create_experiment(name)
    return exp.experiment_id
