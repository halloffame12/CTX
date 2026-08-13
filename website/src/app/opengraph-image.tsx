import { ImageResponse } from "next/og";

export const dynamic = "force-static";

export const alt = "ctx — codebase context for AI coding agents";
export const size = { width: 1200, height: 630 };
export const contentType = "image/png";

export default function OpengraphImage() {
  return new ImageResponse(
    (
      <div
        style={{
          width: "100%",
          height: "100%",
          display: "flex",
          flexDirection: "column",
          justifyContent: "space-between",
          backgroundColor: "#0f172a",
          color: "#e2e8f0",
          padding: 64,
          fontFamily: "ui-sans-serif, system-ui, sans-serif",
        }}
      >
        <div style={{ display: "flex", alignItems: "center", gap: 16 }}>
          <div
            style={{
              display: "flex",
              width: 48,
              height: 48,
              borderRadius: 10,
              backgroundColor: "#0d9488",
              color: "#ffffff",
              alignItems: "center",
              justifyContent: "center",
              fontFamily: "monospace",
              fontSize: 26,
              fontWeight: 700,
            }}
          >
            &gt;_
          </div>
          <div
            style={{
              fontFamily: "monospace",
              fontSize: 30,
              fontWeight: 700,
              color: "#f8fafc",
            }}
          >
            ctx
          </div>
        </div>

        <div
          style={{
            display: "flex",
            flexDirection: "column",
            gap: 8,
          }}
        >
          <div
            style={{
              fontSize: 56,
              fontWeight: 700,
              letterSpacing: "-0.02em",
              lineHeight: 1.1,
              color: "#f8fafc",
            }}
          >
            Codebase context
          </div>
          <div
            style={{
              fontSize: 56,
              fontWeight: 700,
              letterSpacing: "-0.02em",
              lineHeight: 1.1,
              color: "#f8fafc",
            }}
          >
            for AI coding agents
          </div>
        </div>

        <div
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
            fontSize: 22,
            color: "#94a3b8",
          }}
        >
          <span>Symbol search · impact analysis · ranked context</span>
          <span
            style={{
              fontFamily: "monospace",
              fontSize: 20,
              color: "#5eead4",
            }}
          >
            ctx mcp
          </span>
        </div>
      </div>
    ),
    { ...size }
  );
}