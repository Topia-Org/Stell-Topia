from enum import Enum
from typing import Any, Optional

from pydantic import BaseModel, Field, field_validator


class SortBy(str, Enum):
    price = "price"
    duration = "duration"
    departure = "departure"


class SortOrder(str, Enum):
    asc = "asc"
    desc = "desc"


class SearchRequest(BaseModel):
    from_code: str = Field(..., min_length=3, max_length=3, alias="from")
    to_code: str = Field(..., min_length=3, max_length=3, alias="to")
    departure_date: str = Field(...)
    return_date: Optional[str] = None
    passengers: int = Field(..., ge=1, le=9)
    sort_by: SortBy = SortBy.price
    sort_order: SortOrder = SortOrder.asc

    @field_validator("from_code", "to_code", mode="before")
    @classmethod
    def uppercase_codes(cls, value: str) -> str:
        return value.upper()


class Flight(BaseModel):
    id: str
    airline: str
    from_code: str
    to_code: str
    departure_time: str
    arrival_time: str
    duration_minutes: int
    stops: int
    price_usd: float
    price_xlm: float
    seats_available: int


class SearchResponse(BaseModel):
    data: list[Flight]
    meta: dict[str, Any]


class Token(BaseModel):
    access_token: str
    token_type: str
    expires_in: int


class LoginRequest(BaseModel):
    username: str
    password: str




