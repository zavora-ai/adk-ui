import type { KitManifest } from "@zavora-ai/adk-ui-react";

// Two approved brand kits. A host applies a kit to its own pages; the drawing carries no theme,
// and no kit restyles a decision.
export const kits: KitManifest[] = [
  {
    id: "zavora.ai:adk-ui/kit/harbour@1.0.0",
    status: "approved",
    provenance: { reviewedBy: "brand-team", approvedAt: "2026-11-16" },
    tokens: {
      colors: {
        primary: "#0e7490",
        background: "#f8fafc",
        surface: "#ffffff",
        surfaceForeground: "#0f172a",
        muted: "#eef2f6",
        mutedForeground: "#475569",
        border: "#d8e0e8",
        ring: "#0e7490",
        chart: ["#0e7490", "#7c3aed", "#65a30d", "#db2777", "#2563eb", "#0d9488", "#475569", "#a21caf"],
      },
      typography: { bodyFamily: "Inter", monoFamily: "JetBrains Mono" },
      radius: "md",
    },
  },
  {
    id: "zavora.ai:adk-ui/kit/midnight@1.0.0",
    status: "approved",
    provenance: { reviewedBy: "brand-team", approvedAt: "2026-11-16" },
    tokens: {
      colors: {
        primary: "#a78bfa",
        background: "#0b1220",
        surface: "#131c2e",
        surfaceForeground: "#f1f5f9",
        muted: "#1d2940",
        mutedForeground: "#a7b4c6",
        border: "#2a3a57",
        ring: "#a78bfa",
        chart: ["#a78bfa", "#2dd4bf", "#60a5fa", "#f472b6", "#a3e635", "#22d3ee", "#94a3b8", "#e879f9"],
      },
      typography: { bodyFamily: "Inter", monoFamily: "JetBrains Mono" },
      radius: "lg",
    },
  },
];
