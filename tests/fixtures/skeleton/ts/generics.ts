export interface Paginated<T> {
  items: T[];
  next: string | null;
}
export type Maybe<T> = T | null | undefined;
export function paginate<T, K extends keyof T>(rows: T[], key: K): Paginated<T> {
  const items = rows.slice(0, 10);
  return { items, next: rows.length > 10 ? "page2" : null };
}
export const memoize = <T,>(fn: (a: T) => T) => {
  const cache = new Map<T, T>();
  return (a: T): T => {
    const hit = cache.get(a);
    if (hit !== undefined) return hit;
    const v = fn(a);
    cache.set(a, v);
    return v;
  };
};
