import { useEffect, useState } from "react";
import { DrawnView } from "@zavora-ai/adk-surface-react";
import type { DrawnSurface } from "@zavora-ai/adk-surface-react";
import { KitProvider, adkUiRenderers } from "@zavora-ai/adk-ui-react";
import { kits } from "./kits";

type Mode = "home" | "rich" | number;

/**
 * A host of your own: the eleven sample drawings as the ADK-Rust home projected them
 * (`public/surfaces.json`, written by adk-ui's `home_tests`), drawn by `DrawnView` with the
 * home's own view, with adk-ui's renderers, or with those under an approved brand kit. Sending a
 * form here only logs it: a host of your own posts `submitDrawn` to its engine, which asks the
 * person before any tool runs.
 */
export function App() {
  const [surfaces, setSurfaces] = useState<DrawnSurface[]>([]);
  const [mode, setMode] = useState<Mode>("rich");
  const [sent, setSent] = useState<string>();

  useEffect(() => {
    void fetch("./surfaces.json")
      .then((response) => response.json())
      .then(setSurfaces);
  }, []);

  const send = async (actionId: string, values: Record<string, unknown>) => {
    setSent(`${actionId} ${JSON.stringify(values)}: a real host posts submitDrawn, and the person decides on the home's card`);
  };

  const drawings = (
    <div className="host-drawings">
      {surfaces.map((surface) => (
        <DrawnView
          key={surface.surfaceId}
          surface={surface}
          onSend={send}
          renderers={mode === "home" ? undefined : adkUiRenderers}
        />
      ))}
    </div>
  );

  return (
    <main className="host">
      <header className="host-header">
        <h1>adk-ui custom host</h1>
        <nav aria-label="How to draw">
          <button type="button" aria-pressed={mode === "home"} onClick={() => setMode("home")}>
            The home's view
          </button>
          <button type="button" aria-pressed={mode === "rich"} onClick={() => setMode("rich")}>
            adk-ui renderers
          </button>
          {kits.map((kit, index) => (
            <button key={kit.id} type="button" aria-pressed={mode === index} onClick={() => setMode(index)}>
              {kit.id.split("/").pop()}
            </button>
          ))}
        </nav>
        {sent && <p className="host-sent">{sent}</p>}
      </header>
      {typeof mode === "number" ? <KitProvider kit={kits[mode]}>{drawings}</KitProvider> : drawings}
    </main>
  );
}
