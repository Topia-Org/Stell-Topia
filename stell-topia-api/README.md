# Stell-Topia Fare API

FastAPI service that aggregates external fare and inventory sources into a single searchable feed for the Stell-Topia flight-booking protocol.

## Setup

```bash
cd stell-topia-api
pip install -r requirements.txt
uvicorn main:app --reload
```

## Features

- JWT authentication with bcrypt password hashing
- In-memory caching for flight search results (configurable TTL)
- Request ID and processing time middleware for tracing
- Global exception handler with structured error responses
- Search result sorting by price, duration, or departure time

## Endpoints

| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/v1/auth/login` | Login and obtain JWT access token |
| POST | `/api/v1/auth/logout` | Logout (client-side token discard) |
| GET | `/api/v1/health` | Liveness / readiness check |
| POST | `/api/v1/flights/search` | Search available flights |

## Response Headers

Every response includes:
- `x-request-id` — unique request identifier
- `x-process-time-ms` — server processing time in milliseconds

## Login

```bash
curl -X POST http://localhost:8000/api/v1/auth/login \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "username=demo@example.com&password=demopass"
```

Use the returned `access_token` as a Bearer token:

```bash
curl http://localhost:8000/api/v1/flights/search \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{"from":"JFK","to":"LHR","departure_date":"2025-01-15","passengers":1}'
```

## Search Request

```json
{
  "from": "JFK",
  "to": "LHR",
  "departure_date": "2025-01-15",
  "return_date": "2025-01-22",
  "passengers": 1,
  "sort_by": "price",
  "sort_order": "asc"
}
```

## Search Response

```json
{
  "data": [
    {
      "id": "SA742",
      "airline": "Stellar Airways",
      "from_code": "JFK",
      "to_code": "LHR",
      "departure_time": "2025-01-15T10:00:00",
      "arrival_time": "2025-01-15T22:15:00",
      "duration_minutes": 435,
      "stops": 0,
      "price_usd": 450.0,
      "price_xlm": 4090.91,
      "seats_available": 12
    }
  ],
  "meta": {
    "count": 1,
    "currency": "USD",
    "xlmRate": 0.11,
    "providers": ["mock_a", "mock_b", "mock_c"]
  }
}
```

## Environment

| Variable | Default | Description |
|----------|---------|-------------|
| `XLM_TO_USD_RATE` | `0.11` | Conversion rate for XLM pricing |
| `EXTERNAL_PROVIDERS` | `mock_a,mock_b,mock_c` | Comma-separated provider names |
| `SECRET_KEY` | `changeme` | JWT signing secret |
| `JWT_ALGORITHM` | `HS256` | JWT signing algorithm |
| `ACCESS_TOKEN_EXPIRE_MINUTES` | `60` | JWT expiry in minutes |
| `CACHE_TTL_SECONDS` | `120` | In-memory cache TTL for search results |

## Running Tests

```bash
pytest
```

## Lint and Type Check

```bash
ruff check .
mypy app
```




