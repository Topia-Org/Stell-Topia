import time
import uuid
from collections.abc import Callable
from typing import Any, cast

from fastapi import Request, Response
from starlette.middleware.base import BaseHTTPMiddleware


class RequestIdMiddleware(BaseHTTPMiddleware):
    async def dispatch(self, request: Request, call_next: Callable[[Request], Any]) -> Response:
        request_id = request.headers.get("x-request-id", str(uuid.uuid4()))
        request.state.request_id = request_id
        start = time.perf_counter()
        response = cast(Response, await call_next(request))
        duration_ms = int((time.perf_counter() - start) * 1000)
        response.headers["x-request-id"] = request_id
        response.headers["x-process-time-ms"] = str(duration_ms)
        return response



