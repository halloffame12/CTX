export interface Product {
  id: string;
  name: string;
  price: number;
  inStock: boolean;
}

const products: Product[] = [
  { id: "p1", name: "T-Shirt", price: 19.99, inStock: true },
  { id: "p2", name: "Mug", price: 12.5, inStock: true },
  { id: "p3", name: "Sticker", price: 3.0, inStock: false },
];

export async function loadProduct(id: string): Promise<Product | null> {
  return products.find((p) => p.id === id) ?? null;
}

export async function listProducts(): Promise<Product[]> {
  return products;
}