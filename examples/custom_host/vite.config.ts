import { resolve } from "node:path";
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// `@zavora-ai/adk-surface-react` is published with ADK-Rust 3.0. Until then this host builds it
// from an adk-rust checkout: ADK_SURFACE_ROOT is that checkout's `adk-surface` directory.
const surface = resolve(process.env.ADK_SURFACE_ROOT ?? resolve(__dirname, "../../../adk-rust/adk-surface"));

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: [
      { find: "@zavora-ai/adk-surface-react/tokens.css", replacement: resolve(surface, "web/src/styles/tokens.css") },
      { find: "@zavora-ai/adk-surface-react", replacement: resolve(surface, "react/src/index.ts") },
      { find: "@zavora-ai/adk-ui-react/style.css", replacement: resolve(__dirname, "../../packages/adk-ui-react/src/styles.css") },
      { find: "@zavora-ai/adk-ui-react", replacement: resolve(__dirname, "../../packages/adk-ui-react/src/index.ts") },
    ],
    // One React for the host, the home's renderer and adk-ui's renderers.
    dedupe: ["react", "react-dom", "recharts", "three"],
  },
  server: { fs: { allow: [surface, resolve(__dirname, "../..")] } },
});
