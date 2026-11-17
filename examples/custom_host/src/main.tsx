import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "@zavora-ai/adk-surface-react/tokens.css";
import "@zavora-ai/adk-ui-react/style.css";
import "./app.css";
import { App } from "./App";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
