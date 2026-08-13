from fastapi import FastAPI, Request
from fastapi.responses import JSONResponse

from app.config import settings
from app.middleware import RequestIdMiddleware
from app.routers import auth, flights

app = FastAPI(
    title=settings.app_name,
    version=settings.app_version,
)

app.add_middleware(RequestIdMiddleware)


@app.exception_handler(Exception)
async def global_exception_handler(request: Request, exc: Exception) -> JSONResponse:
    return JSONResponse(
        status_code=500,
        content={
            "error": {
                "code": "internal_error",
                "message": "An unexpected error occurred",
            }
        },
        headers={"x-request-id": getattr(request.state, "request_id", "")},
    )


app.include_router(flights.router, prefix=settings.api_prefix)
app.include_router(auth.router, prefix=settings.api_prefix)



