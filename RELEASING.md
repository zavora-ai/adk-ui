# Releasing

adk-ui and `@zavora-ai/adk-ui-react` share one version. Publish both from `main` and tag the
commit `vX.Y.Z`. The 2.x line is published from `release/2.x`.

## 3.0.0

adk-ui 3.0.0 builds on crates that aren't published yet, so the order matters.

1. **adk-rust.** genui's scenes and depth (zavora-ai/adk-rust#1449), the axis-label fix
   (#1447) and the extension-catalog docs (#1448) are merged. adk-ui 3 also needs genui's
   2026-11 work on branch `feat/genui-sota`, not yet opened as a pull request: A2UI 1.0 on
   the wire and catalogs as A2UI 1.0 documents (`Catalog::with_instructions`,
   `document_for`, `components_in`), `drawn_tree` (`show_drawing`, `eval`), dark-pattern
   warnings (the fixtures test), and an origin per MCP server (`apps_tests`). It is rebased
   onto `main`; open its pull request and merge it first (rebase again if `main` moved).
2. **ADK-Rust 3.0.0.** Wait until it's on crates.io (every `adk-*` crate in `Cargo.toml`) and
   `@zavora-ai/adk-surface-react` 3.0.0 is on npm. If that work ships after 3.0.0, raise
   `adk-surface` in `Cargo.toml` to the first release that has it.
3. **Build against the registry**, from a clean checkout of `main`:

   ```bash
   rm -f .cargo/config.toml                 # drop the local adk-rust patch
   cargo update                             # lockfile from crates.io
   cargo fmt --check
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test --all-features
   cargo run --example drawing_eval         # every sample drawing shown
   RUSTDOCFLAGS="--cfg docsrs" cargo +nightly doc --no-deps --all-features
   cargo publish --dry-run
   cargo package --list                     # src/, catalogs/, three docs, README, CHANGELOG, LICENSE
   ```

4. **The React package:**

   ```bash
   npm ci
   cd packages/adk-ui-react
   npm run lint && npm run typecheck && npm test
   npm pack --dry-run                       # dist/, README.md and LICENSE
   ```

5. **The custom host.** Replace the source aliases in `examples/custom_host/vite.config.ts`
   and `tsconfig.json` with the published packages, add
   `import "@zavora-ai/adk-surface-react/style.css"` to `src/main.tsx` (the source alias
   brought it in), then run `npm install && npm run typecheck && npm run build` there.
6. **Date the release.** In `CHANGELOG.md`, turn `## [Unreleased] — 3.0.0` into
   `## [3.0.0] - YYYY-MM-DD`, delete `.cargo/config.toml.example` and the "build against an
   unreleased adk-rust" notes (README "Developing", the comment in `Cargo.toml`), then commit
   and tag `v3.0.0`.
7. **Publish:**

   ```bash
   cargo publish
   cd packages/adk-ui-react && npm publish  # prepublishOnly lints, type-checks and tests
   ```

8. **GitHub release.** Create the `v3.0.0` release with `docs/releases/v3.0.0.md` as its notes.
9. **adk-rust follow-ups.** In spec 3.3, mark the scenes-and-depth amendment decided, and point
   the genui guide at the published crate.

## 2.2.1 (`release/2.x`)

The 2.x line takes security fixes only, until the `adk_ui` server profile's sunset on
2026-12-31.

```bash
git switch release/2.x
# CHANGELOG.md: date 2.2.1
cargo publish
cd packages/adk-ui-react && npm publish --tag 2.x
```

If 3.0.0 is already out, publish with `--tag 2.x`: a plain `npm publish` would move `latest`
back to 2.2.1.
