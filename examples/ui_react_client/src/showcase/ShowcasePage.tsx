import { useEffect, useRef, useState } from 'react';
import {
  AdkUiKitProvider,
  ApplicationRenderer,
  StreamingRenderer,
  type UiEvent,
} from '@zavora-ai/adk-ui-react';
import { findShowcaseExample, SHOWCASE_EXAMPLES } from './catalog';
import type { ShowcaseEventReceipt, StreamingShowcaseExample } from './types';
import './showcase.css';

function eventReceipt(event: UiEvent): ShowcaseEventReceipt {
  if (event.action === 'form_submit') {
    return {
      label: `Form submitted: ${event.action_id}`,
      detail: `${Object.keys(event.data).length} structured fields emitted to the host.`,
      event,
    };
  }
  if (event.action === 'input_change') {
    return { label: `Input changed: ${event.name}`, detail: String(event.value), event };
  }
  if (event.action === 'tab_change') {
    return { label: 'Tab changed', detail: `Selected tab ${event.index + 1}.`, event };
  }
  return { label: `Action emitted: ${event.action_id}`, detail: 'The host received a structured button event.', event };
}

function StreamingExampleSurface({
  example,
  onAction,
}: {
  example: StreamingShowcaseExample;
  onAction: (event: UiEvent) => void;
}) {
  const [step, setStep] = useState(0);
  const [run, setRun] = useState(0);

  useEffect(() => {
    if (step >= example.updates.length) return;
    const timer = window.setTimeout(() => setStep((current) => current + 1), step === 0 ? 650 : 900);
    return () => window.clearTimeout(timer);
  }, [example.updates.length, run, step]);

  return (
    <AdkUiKitProvider kit={example.kit}>
      <div className="showcase-stream">
        <div className="showcase-stream__status">
          <span><i /> Update {step} / {example.updates.length}</span>
          <button
            type="button"
            onClick={() => {
              setStep(0);
              setRun((current) => current + 1);
            }}
          >
            Replay stream
          </button>
        </div>
        <StreamingRenderer
          key={run}
          component={example.initialComponent}
          updates={step > 0 ? example.updates[step - 1] : undefined}
          onAction={onAction}
        />
      </div>
    </AdkUiKitProvider>
  );
}

function ShowcaseNotFound({ requestedId }: { requestedId: string }) {
  return (
    <main className="showcase-missing">
      <p>Example not found</p>
      <h1>{requestedId}</h1>
      <a href="./#showcase">Return to the showcase</a>
    </main>
  );
}

export function ShowcasePage({ exampleId }: { exampleId: string }) {
  const example = findShowcaseExample(exampleId);
  const canvasRef = useRef<HTMLElement | null>(null);
  const pendingNavigationReceiptRef = useRef<ShowcaseEventReceipt | null>(null);
  const [viewport, setViewport] = useState<'desktop' | 'mobile'>('desktop');
  const [isFullscreen, setIsFullscreen] = useState(false);
  const [receipt, setReceipt] = useState<ShowcaseEventReceipt>({
    label: 'Example ready',
    detail: 'Interact with the generated UI to inspect the event boundary.',
  });

  useEffect(() => {
    const syncFullscreen = () => setIsFullscreen(canvasRef.current?.matches(':fullscreen') ?? false);
    document.addEventListener('fullscreenchange', syncFullscreen);
    return () => document.removeEventListener('fullscreenchange', syncFullscreen);
  }, []);

  if (!example) return <ShowcaseNotFound requestedId={exampleId} />;

  const index = SHOWCASE_EXAMPLES.indexOf(example);
  const previous = SHOWCASE_EXAMPLES[(index - 1 + SHOWCASE_EXAMPLES.length) % SHOWCASE_EXAMPLES.length];
  const next = SHOWCASE_EXAMPLES[(index + 1) % SHOWCASE_EXAMPLES.length];
  const sourceUrl = `https://github.com/zavora-ai/adk-ui/blob/main/examples/ui_react_client/src/showcase/examples/${example.id}.ts`;

  async function toggleFullscreen() {
    try {
      if (document.fullscreenElement) {
        await document.exitFullscreen();
      } else {
        await canvasRef.current?.requestFullscreen();
      }
    } catch (error) {
      setReceipt({
        label: 'Fullscreen unavailable',
        detail: error instanceof Error ? error.message : 'The host browser rejected the request.',
      });
    }
  }

  function handleAction(event: UiEvent) {
    const nextReceipt = eventReceipt(event);
    pendingNavigationReceiptRef.current =
      (event.action === 'button_click' || event.action === 'form_submit')
      && event.action_id.startsWith('navigate:')
        ? nextReceipt
        : null;
    setReceipt(nextReceipt);
  }

  function handleNavigate(route: string, page: { title: string }) {
    const navigationAction = pendingNavigationReceiptRef.current;
    pendingNavigationReceiptRef.current = null;
    setReceipt(navigationAction
      ? {
          ...navigationAction,
          label: `${navigationAction.label} and navigated`,
          detail: `${navigationAction.detail} Local route: ${route}.`,
        }
      : { label: `Navigated to ${page.title}`, detail: `Local application route: ${route}` });
  }

  return (
    <div
      className="showcase-page"
      data-showcase-id={example.id}
      style={{ '--showcase-accent': example.accent } as React.CSSProperties}
    >
      <header className="showcase-page__header">
        <a className="showcase-page__brand" href="./#showcase">
          <span>ADK UI</span>
          <i>Showcase</i>
        </a>
        <div className="showcase-page__switcher">
          <a href={`?example=${previous.id}`} aria-label={`Previous example: ${previous.title}`}>&lt;</a>
          <span>{example.number} / {String(SHOWCASE_EXAMPLES.length).padStart(2, '0')}</span>
          <a href={`?example=${next.id}`} aria-label={`Next example: ${next.title}`}>&gt;</a>
        </div>
      </header>

      <section className="showcase-page__brief" aria-labelledby="showcase-example-title">
        <div>
          <p>{example.eyebrow}</p>
          <h1 id="showcase-example-title">{example.title}</h1>
        </div>
        <p>{example.description}</p>
        <div className="showcase-page__protocols">
          {example.protocols.map((protocol) => <span key={protocol}>{protocol}</span>)}
        </div>
      </section>

      <div className="showcase-page__capabilities" aria-label="Capabilities demonstrated">
        {example.capabilities.map((capability) => <span key={capability}>{capability}</span>)}
        <a href={sourceUrl} target="_blank" rel="noreferrer">View source</a>
      </div>

      <div className="showcase-page__toolbar">
        <div role="group" aria-label="Example viewport">
          <button type="button" aria-pressed={viewport === 'desktop'} onClick={() => setViewport('desktop')}>Desktop</button>
          <button type="button" aria-pressed={viewport === 'mobile'} onClick={() => setViewport('mobile')}>Mobile</button>
        </div>
        <button type="button" disabled={!document.fullscreenEnabled} onClick={() => void toggleFullscreen()}>
          {isFullscreen ? 'Exit fullscreen' : 'Fullscreen'}
        </button>
      </div>

      <section ref={canvasRef} className={`showcase-page__canvas showcase-page__canvas--${viewport}`} aria-label={`${example.title} generated interface`}>
        <div className="showcase-page__viewport">
          {example.kind === 'application' ? (
            <ApplicationRenderer
              application={example.application}
              kits={[example.kit]}
              fallbackKit={example.kit}
              onAction={handleAction}
              onNavigate={handleNavigate}
            />
          ) : (
            <StreamingExampleSurface example={example} onAction={handleAction} />
          )}
        </div>
      </section>

      <aside className="showcase-page__receipt" aria-live="polite">
        <span><i /> Host event</span>
        <strong>{receipt.label}</strong>
        <p>{receipt.detail}</p>
      </aside>
    </div>
  );
}
