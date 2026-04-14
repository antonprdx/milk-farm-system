from pydantic_settings import BaseSettings


class Settings(BaseSettings):
    database_url: str = "postgresql+asyncpg://milkfarm:milkfarm@db:5432/milkfarm"
    model_dir: str = "/app/models"
    retrain_cron_hour: int = 3

    model_config = {"env_file": ".env", "env_file_encoding": "utf-8"}


settings = Settings()
