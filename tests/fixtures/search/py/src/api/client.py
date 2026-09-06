from src.models.user import User, UserRepository


class ApiClient:
    def __init__(self, repo: UserRepository):
        self._repo = repo

    def fetch_user(self, user_id: int) -> User:
        return self._repo.find_by_id(user_id)