import type { Metadata } from "next";
import Playground from "./playground-client";

export const metadata: Metadata = {
  title: "Playground — CTX",
  description:
    "Try CTX queries in the browser: search, symbol, deps, impact, context, and skeleton against a bundled sample TypeScript shop.",
};

export default function PlaygroundPage() {
  return <Playground />;
}