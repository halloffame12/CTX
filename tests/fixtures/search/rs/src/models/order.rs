pub struct Order<T> {
    pub id: u64,
    pub items: Vec<T>,
}

impl<T> Order<T> {
    pub fn total(&self) -> u64 {
        self.items.len() as u64
    }
}

pub type OrderId = u64;

pub fn make_order<T>(items: Vec<T>) -> Order<T> {
    Order { id: 0, items }
}