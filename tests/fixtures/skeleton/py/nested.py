"""Nested functions, closures, async, decorators."""
import asyncio

def outer(a: int) -> int:
    cache: dict = {}

    def inner(b: int) -> int:
        return cache.get(b, a + b)

    return inner(2)

@asyncio.coroutine
def legacy_coro() -> int:
    yield 1

async def fetch_all(urls: list[str]) -> list[str]:
    async def fetch_one(url: str) -> str:
        resp = await http_get(url)
        return resp.body

    results = []
    for u in urls:
        results.append(await fetch_one(u))
    return results
