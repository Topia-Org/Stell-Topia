from pydantic_settings import BaseSettings


class Settings(BaseSettings):
    app_name: str = "Stell-Topia Fare API"
    app_version: str = "0.1.0"
    api_prefix: str = "/api/v1"
    xlm_to_usd_rate: float = 0.11
    external_providers: list[str] = ["mock_a", "mock_b", "mock_c"]
    secret_key: str = "changeme"
    jwt_algorithm: str = "HS256"
    access_token_expire_minutes: int = 60
    cache_ttl_seconds: int = 120

    class Config:
        env_file = ".env"


settings = Settings()



