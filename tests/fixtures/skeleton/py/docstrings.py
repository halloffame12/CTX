"""Module docstring stays."""


def documented(a: int) -> int:
    """Sums with one.

    Detailed explanation that is long.
    """
    return a + 1


class Config:
    """Config class docstring."""

    defaults = {"port": 8080}

    def load(self, path):
        """Load config from path."""
        import json

        with open(path) as f:
            return json.load(f)