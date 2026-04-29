from fastapi import Depends, HTTPException, Request, Security
from fastapi.security import APIKeyHeader

from app.config import settings

_api_key_header = APIKeyHeader(name="X-API-Key", auto_error=False)


async def verify_api_key(
    request: Request,
    api_key: str | None = Security(_api_key_header),
) -> None:
    if not settings.api_key:
        return

    if api_key == settings.api_key:
        return

    raise HTTPException(status_code=401, detail="Invalid or missing API key")
