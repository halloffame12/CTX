import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";
import "./globals.css";

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  metadataBase: new URL("https://halloffame12.github.io/CTX/"),
  title: "ctx — Codebase intelligence & context engine for AI agents",
  description:
    "ctx indexes your codebase into a local, searchable graph and produces relevance-ranked context packages for AI coding agents. Fast, private, offline, deterministic.",
  keywords: [
    "ctx",
    "code intelligence",
    "AI coding agents",
    "code graph",
    "MCP",
    "context engine",
    "code search",
    "symbol search",
  ],
  openGraph: {
    title: "ctx — Codebase intelligence & context engine for AI agents",
    description:
      "Local, fast, private code graph + ranked context packages for AI coding agents. MCP over stdio.",
    type: "website",
    url: "https://halloffame12.github.io/CTX/",
  },
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html
      lang="en"
      className={`${geistSans.variable} ${geistMono.variable} h-full antialiased`}
    >
      <body className="min-h-full flex flex-col bg-ink-950 text-foreground">
        {children}
      </body>
    </html>
  );
}