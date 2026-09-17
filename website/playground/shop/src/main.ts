import { Cart, checkout, emptyCart, addProduct } from "../cart/cart";
import { applyPromo } from "../cart/promo";
import { login } from "../auth/login";
import { listProducts, loadProduct } from "../catalog/products";
import { formatMoney, parseMoney } from "../money";

const cart: Cart = emptyCart("cart-1", "USD");

export async function runShop(): Promise<string> {
  await addProduct(cart, "p1", 2);
  await addProduct(cart, "p2", 1);

  const session = await login({ email: "ada@example.com", password: "x" });
  if (!session.ok) return "login failed";

  const products = await listProducts();
  const first = await loadProduct(products[0].id);
  const price = first ? formatMoney(first.price, "USD") : "$0.00";
  parseMoney(price, "USD");

  const discount = applyPromo("SAVE10", 42);
  await checkout(cart, "SAVE10", { print: () => {} });

  return `total ${discount}`;
}