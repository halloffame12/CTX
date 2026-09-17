import { Printer, PrinterRequest, printReceipt } from "../payments/receipt";
import { applyPromo, PromoCode } from "./promo";
import { loadProduct, Product } from "../catalog/products";
import { Currency } from "../money";

export interface CartLine {
  productId: string;
  quantity: number;
}

export interface Cart {
  id: string;
  lines: CartLine[];
  currency: Currency;
}

export function emptyCart(id: string, currency: Currency): Cart {
  return { id, lines: [], currency };
}

export async function addProduct(
  cart: Cart,
  productId: string,
  quantity = 1,
): Promise<void> {
  const product = await loadProduct(productId);
  if (!product) throw new Error(`unknown product ${productId}`);
  cart.lines.push({ productId, quantity });
}

export async function checkout(
  cart: Cart,
  promo: PromoCode | null,
  printer: Printer,
): Promise<PrinterRequest> {
  const items: Product[] = [];
  for (const line of cart.lines) {
    const product = await loadProduct(line.productId);
    if (product) items.push(product);
  }
  const discount = promo ? applyPromo(promo, total(cart)) : 0;
  return printReceipt(printer, items, discount, cart.currency);
}

export function total(cart: Cart): number {
  return cart.lines.reduce((sum, line) => {
    return sum + line.quantity;
  }, 0);
}