# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.6.0] - 2026-04-17

### Changed

- Upgraded all `adk-*` dependencies to 0.6.0 (adk-core, adk-agent, adk-model, adk-cli, adk-runner, adk-server, adk-session).
- Updated example server `FunctionResponseData` initialization to include new `file_data` and `inline_data` fields from adk-core 0.6.

## [0.5.0] - 2026-03-30

### Changed

- Upgraded all `adk-*` dependencies to 0.5.0 (adk-core, adk-agent, adk-model, adk-cli, adk-runner, adk-server, adk-session).
- Migrated `AdkError::Tool(...)` enum variant to `AdkError::tool(...)` constructor across all render tools, protocol output, interop adapters, and MCP Apps validation to match the adk-core 0.5 API.
- Updated example server `runner.run()` to use typed `UserId` and `SessionId` parameters.
- Revised README with professional structure, consistent formatting, and improved scannability.

### Added

- `AdkError::tool()` constructor on the standalone compat fallback for parity with adk-core 0.5.
- Root `package.json` for npm workspace support across the React client and shared renderer package.
- Verification screenshot for A2UI dashboard render.

### Fixed

- Compilation under `--features adk-core` against adk-core 0.5.0.

## [0.4.0] - 2026-03-20

### Added

- Initial public release with 30 component types and 13 high-level render tools.
- Protocol support for A2UI (v0.9 draft-aligned), AG-UI (stable 0.1 subset), MCP Apps (SEP-1865 subset), and legacy adk_ui.
- Full render tool × protocol matrix coverage (39/39 combinations tested).
- A2UI validator with JSON Schema-based message validation.
- Interop adapters for A2UI JSONL, AG-UI event streams, and MCP Apps bridge payloads.
- React reference client (`@zavora-ai/adk-ui-react`) with protocol profile selector.
- Rust example server with SSE streaming and multi-agent demo apps.
- Runtime capability metadata exposed via `/api/ui/capabilities`.
- Protocol migration guide and deprecation timeline for legacy `adk_ui` profile.
