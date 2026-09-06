from typing import Generic, TypeVar, Optional, Callable, Dict, List

T = TypeVar("T")

def transform(items: List[T], fn: Callable[[T], T]) -> List[T]:
    out = [fn(i) for i in items]
    return out

class Registry(Generic[T]):
    def lookup(self, key: str) -> Optional[T]:
        entry = self._store.get(key)
        return entry

async def run(handler: Callable[..., T]) -> Dict[str, T]:
    result = await handler()
    return {"ok": result}
