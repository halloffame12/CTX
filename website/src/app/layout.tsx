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
  metadataBase: new URL("https://halloffame12.github.io/CTX"),
  title: {
    default: "ctx — codebase context for AI coding agents",
    template: "%s · ctx",
  },
  description:
    "ctx indexes a repository into a local, deterministic code graph and answers an agent's questions with real file paths: where a symbol lives, what would break if it changed, and which files a task actually needs.",
  keywords: [
    "mcp server",
    "model context protocol",
    "ai coding agents",
    "code graph",
    "code intelligence",
    "code search",
    "impact analysis",
    "developer tools",
  ],
  authors: [{ name: "Sumit Chauhan", url: "https://github.com/halloffame12" }],
  creator: "Sumit Chauhan",
  publisher: "Sumit Chauhan",
  openGraph: {
    title: "ctx — codebase context for AI coding agents",
    description:
      "A local, deterministic code graph for AI agents: symbol search, impact analysis, and ranked context over stdio.",
    url: "https://halloffame12.github.io/CTX",
    siteName: "ctx",
    type: "website",
  },
  twitter: {
    card: "summary",
    title: "ctx — codebase context for AI coding agents",
    description:
      "A local, deterministic code graph for AI agents: symbol search, impact analysis, and ranked context over stdio.",
  },
  robots: {
    index: true,
    follow: true,
  },
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en" className={`${geistSans.variable} ${geistMono.variable}`}>
      <body>{children}</body>
    </html>
  );
}