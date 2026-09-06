from __future__ import annotations
import os
from typing import Optional, TypeVar
from .db import Session
from .models import User

T = TypeVar("T")

class Repository(Generic[T]):
    def __init__(self, session: Session) -> None:
        self._session = session

    def find(self, pk: int) -> Optional[T]:
        row = self._session.get(pk)
        return row

    def save(self, obj: T) -> T:
        self._session.add(obj)
        return obj

@dataclass
class UserProfile:
    name: str
    email: str

def build_repository(session: Session) -> Repository[User]:
    return Repository[User](session)


def _helper(x: int) -> int:
    return x + 1
