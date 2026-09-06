def format_date(value):
    return str(value)


def format_currency(amount: float, currency: str = "USD") -> str:
    return f"{currency} {amount:.2f}"


INTERNAL = "internal"
VERSION = "1.0.0"


def _helper(x):
    return x