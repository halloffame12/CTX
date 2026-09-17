export type Currency = "USD" | "EUR" | "CAD";

export interface Money {
  amount: number;
  currency: Currency;
}

export function formatMoney(value: number, currency: Currency): string {
  const symbol = currency === "EUR" ? "€" : currency === "CAD" ? "C$" : "$";
  return `${symbol}${value.toFixed(2)}`;
}

export function parseMoney(value: string, currency: Currency): Money {
  const amount = Number.parseFloat(value.replace(/[^0-9.-]/g, ""));
  return { amount, currency };
}