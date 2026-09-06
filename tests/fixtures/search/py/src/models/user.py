from datetime import datetime

DEFAULT_ROLE = "user"
ADMIN_ROLE = "admin"


class User:
    def __init__(self, name: str, email: str):
        self.name = name
        self.email = email

    def display_name(self) -> str:
        return f"{self.name} <{self.email}>"

    def _private(self) -> None:
        pass


class UserRepository:
    def __init__(self, users=None):
        self.users = users or []

    def find_by_id(self, user_id: int):
        for u in self.users:
            if u.id == user_id:
                return u
        raise KeyError(user_id)


def create_user(name: str, email: str) -> User:
    return User(name, email)