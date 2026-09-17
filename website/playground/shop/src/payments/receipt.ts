import { Product } from "../catalog/products";
import { Currency } from "../money";

export interface PrinterRequest {
  lines: string[];
}

export interface Printer {
  print(req: PrinterRequest): void;
}

export async function printReceipt(
  printer: Printer,
  items: Product[],
  discount: number,
  currency: Currency,
): Promise<PrinterRequest> {
  const lines = [];
  for (const item of items) {
    lines.push(`${item.name} — ${item.price} ${currency}`);
  }
  if (discount > 0) {
    lines.push(`discount −${discount} ${currency}`);
  }
  printer.print({ lines });
  return { lines };
}