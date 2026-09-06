from enum import Enum
from dataclasses import dataclass, field
from typing import List


class OrderStatus(Enum):
    PENDING = "pending"
    PAID = "paid"
    SHIPPED = "shipped"


@dataclass
class Order:
    id: int
    status: OrderStatus = OrderStatus.PENDING
    line_items: List[str] = field(default_factory=list)

    def total(self) -> float:
        return 0.0


def make_order(items: List[str]) -> Order:
    return Order(id=1, status=OrderStatus.PAID, line_items=items)


MAX_ITEMS = 100