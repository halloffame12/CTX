class Cart {
  items = [];

  constructor() {
    this.total = 0;
  }

  addItem(item) {
    this.items.push(item);
    this.total += item.price;
  }

  getCount() {
    return this.items.length;
  }
}

module.exports = Cart;