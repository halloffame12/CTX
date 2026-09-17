import { Cart } from "./cart";

export type PromoCode = "SAVE10" | "WELCOME5" | "FREESHIP";

export interface DiscountResult {
  amount: number;
  applied: PromoCode;
}

export function applyPromo(code: PromoCode, subtotal: number): number {
  switch (code) {
    case "SAVE10":
      return round(subtotal * 0.1);
    case "WELCOME5":
      return round(subtotal * 0.05);
    case "FREESHIP":
      return 0;
  }
}

export function applyPromoToCart(
  cart: Cart,
  code: PromoCode,
): DiscountResult {
  const subtotal = cart.lines.reduce((sum, l) => sum + l.quantity, 0);
  return { amount: applyPromo(code, subtotal), applied: code };
}

function round(value: number): number {
  return Math.round(value * 100) / 100;
}