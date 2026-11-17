# A host of your own

The six sample drawings of `fixtures/drawings/`, as the ADK-Rust home projected them
(`public/surfaces.json`, written by `cargo test --test home_tests`), drawn by
`@zavora-ai/adk-surface-react`'s `DrawnView`:

- with the home's own view of every extension component (charts as the home's SVG chart);
- with `@zavora-ai/adk-ui-react`'s renderers (`adkUiRenderers`);
- with those renderers under one of two approved brand kits (`KitProvider`).

```bash
npm install
# Until ADK-Rust 3.0 publishes @zavora-ai/adk-surface-react, point at an adk-rust checkout:
ADK_SURFACE_ROOT=../../../adk-rust/adk-surface npm run dev
npm run typecheck   # tsconfig.json maps @zavora-ai/adk-surface-react to ../../../adk-rust
```

A form's Send only logs here. A real host posts `submitDrawn` to its engine
(`POST /api/surface/commands`), the drawing agent holds the call, and the person decides on the
home's card; see `examples/insight_home.rs` for a home that does.
