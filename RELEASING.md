# Releasing

adk-ui and `@zavora-ai/adk-ui-react` share one version. Publish both from `main` and tag the
commit `vX.Y.Z`. The 2.x line is published from `release/2.x`.

## 3.0.0

adk-ui 3.0.0 builds on crates that aren't published yet, so the order matters.

1. **adk-rust.** Merge `genui`'s scenes and depth effects (zavora-ai/adk-rust#1449), which the
   spatial catalog projects onto. The axis-label fix (#1447) and the extension-catalog docs
   (#1448) are already merged.
2. **ADK-Rust 3.0.0.** Wait until it's on crates.io (every `adk-*` crate in `Cargo.toml`) and
   `@zavora-ai/adk-surface-react` 3.0.0 is on npm. If scenes ship after 3.0.0, raise
   `adk-surface` in `Cargo.toml` to the first release that has them.
3. **Build against the registry**, from a clean checkout of `main`:

   ```bash
   rm -f .cargo/config.toml                 # drop the local adk-rust patch
   cargo update                             # lockfile from crates.io
   cargo fmt --check
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test --all-features
   cargo publish --dry-run
   cargo package --list                     # no .cargo, packages/, custom_host or archive
   ```

4. **The React package:**

   ```bash
   npm ci
   cd packages/adk-ui-react
   npm run lint && npm run typecheck && npm test
   npm pack --dry-run                       # dist/ and README.md only
   ```

5. **The custom host.** Replace the source aliases in `examples/custom_host/vite.config.ts`
   and `tsconfig.json` with the published packages, then run
   `npm install && npm run typecheck && npm run build` there.
6. **Date the release.** In `CHANGELOG.md`, turn `## [Unreleased] — 3.0.0` into
   `## [3.0.0] - YYYY-MM-DD`, then commit and tag `v3.0.0`.
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
