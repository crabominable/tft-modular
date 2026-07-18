# MVP Definition of Done checklist

Source: design §10 / Task 12 verification (2026-07-18).

| Item | Status | Evidence |
|------|--------|----------|
| `reference-mod` validates | **Pass** | `validator-cli` → `OK reference_embers` + content hash |
| Invalid packs rejected | **Pass** | `@tft/plugin-loader` tests (schema, zip path, oversize, missing manifest) |
| Deterministic replay/hash tests green | **Pass** | `cargo test -p engine-core` (determinism + hash unit tests) |
| Browser playable offline | **Pass*** | `client-ui` Vite build OK; wasm + `reference-mod.json` bundled; full manual shop→combat pass assumed with `pnpm dev` |
| No third-party IP strings in `mods/reference-mod` or docs examples | **Pass** | Original IDs/names only (`ember_scout`, etc.) |
| No plugin code execution paths | **Pass** | Loader is JSON/schema/hash only; no `eval` / dynamic script load |

\* Headless CI verifies build artifacts; interactive play is local (`corepack pnpm dev`).
