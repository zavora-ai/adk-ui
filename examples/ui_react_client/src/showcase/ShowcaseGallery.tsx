import type { CSSProperties } from 'react';
import { SHOWCASE_MANIFEST } from './manifest';
import './showcase.css';

function PreviewGlyph({ exampleId }: { exampleId: string }) {
  return (
    <div className="showcase-card__visual" data-visual={exampleId} aria-hidden="true">
      <span className="showcase-card__visual-orb" />
      <span className="showcase-card__visual-line line-a" />
      <span className="showcase-card__visual-line line-b" />
      <span className="showcase-card__visual-panel panel-a" />
      <span className="showcase-card__visual-panel panel-b" />
      <span className="showcase-card__visual-panel panel-c" />
    </div>
  );
}

export function ShowcaseGallery() {
  return (
    <section id="showcase" className="showcase-gallery" aria-labelledby="showcase-heading">
      <header className="showcase-gallery__intro">
        <div>
          <p className="section-kicker">Independent examples</p>
          <h2 id="showcase-heading">See one capability at a time.</h2>
        </div>
        <p>
          Six zero-server examples render through the public package. Open any example as a
          standalone app, interact with it, then inspect the source fixture.
        </p>
      </header>

      <div className="showcase-gallery__grid">
        {SHOWCASE_MANIFEST.map((example, index) => (
          <a
            className={`showcase-card${index === 0 ? ' showcase-card--featured' : ''}`}
            href={`?example=${example.id}`}
            key={example.id}
            style={{ '--showcase-accent': example.accent } as CSSProperties}
          >
            <PreviewGlyph exampleId={example.id} />
            <div className="showcase-card__body">
              <span className="showcase-card__number">{example.number}</span>
              <p>{example.eyebrow}</p>
              <h3>{example.title}</h3>
              <span className="showcase-card__summary">{example.capabilitySummary}</span>
              <div className="showcase-card__tags">
                {example.capabilities.slice(0, 3).map((capability) => <i key={capability}>{capability}</i>)}
              </div>
              <strong>Run example <span aria-hidden="true">-&gt;</span></strong>
            </div>
          </a>
        ))}
      </div>
    </section>
  );
}
