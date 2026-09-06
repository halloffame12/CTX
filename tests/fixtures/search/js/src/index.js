const Cart = require("./models/cart");
const math = require("./utils/math");

async function main() {
  const c = new Cart();
  c.addItem({ name: "widget", price: 10 });
  const total = math.add(1, 2);
  return { c, total };
}

main();