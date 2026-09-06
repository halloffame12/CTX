export async function dyn() {
  const m = await import("./lazy");
  return m.lazy();
}
