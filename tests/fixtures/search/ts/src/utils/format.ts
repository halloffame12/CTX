export function formatDate(d: Date): string {
  return d.toISOString();
}

export function formatNumber(n: number, locale = "en-US"): string {
  return n.toLocaleString(locale);
}

const INTERNAL_PREFIX = "ctx:";

export const VERSION = "1.0.0";

export function parseDate(s: string): Date {
  return new Date(s);
}

function helper(x: number): number {
  return x * 2;
}