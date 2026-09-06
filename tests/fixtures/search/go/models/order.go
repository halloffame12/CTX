package models

type Order struct {
	ID    int64
	Items []string
}

type OrderID = int64

const DefaultCurrency = "USD"

func MakeOrder(id int64) *Order {
	return &Order{ID: id}
}

func (o *Order) Total() int {
	return len(o.Items)
}

func DuplicateOrderID() OrderID {
	return 0
}