# TFT Modular

Offline web auto-battler with **data-only** plugins and a Rust/Wasm rules core.

- Spec: [`docs/superpowers/specs/2026-07-18-tft-modular-mvp-design.md`](docs/superpowers/specs/2026-07-18-tft-modular-mvp-design.md)
- DoD: [`docs/superpowers/plans/mvp-dod-checklist.md`](docs/superpowers/plans/mvp-dod-checklist.md)

## Requirements

| Tool | Notes |
|------|--------|
| **Rust** | Stable toolchain. On Windows, prefer the **gnu** host if MSVC link issues appear: `rustup default stable-x86_64-pc-windows-gnu` |
| **wasm-pack** | `cargo install wasm-pack` |
| **Node.js** | 20+ |
| **pnpm** | Via Corepack: `corepack enable` then use `corepack pnpm` (avoids global shim EPERM on some Windows setups) |

Ensure Cargo binaries are on `PATH` (PowerShell session example):

```powershell
$env:Path = "$env:USERPROFILE\.cargo\bin;" + $env:Path
```

## Dev

From the repo root:

```powershell
# install JS workspace
corepack pnpm install

# verify
cargo test -p engine-core
corepack pnpm --filter @tft/plugin-loader test
corepack pnpm --filter validator-cli start -- ../../mods/reference-mod
# or: corepack pnpm validate-mod

# browser slice (wasm package is gitignored — build it first)
corepack pnpm build:wasm
node --experimental-strip-types tools/bundle-mod.mjs
corepack pnpm dev
```

Open the Vite URL (usually `http://localhost:5173`). Shop → place units → end phase → combat → HUD shows `state_hash`. Offline after first load (bundled `reference-mod.json` + local wasm).

### Useful root scripts

| Script | What it does |
|--------|----------------|
| `corepack pnpm test` | Recursive package tests (loader, etc.) |
| `corepack pnpm validate-mod` | Validator CLI on `mods/reference-mod` |
| `corepack pnpm build:wasm` | `wasm-pack` → `packages/client-ui/src/wasm-pkg` |
| `corepack pnpm dev` | Vite dev server for `client-ui` |
| `cargo test -p engine-core` | Engine unit + determinism + economy tests |

## Layout

```
crates/engine-core     # pure Rust match rules, hash, replay
crates/engine-wasm     # wasm-bindgen bridge
packages/plugin-schema # JSON Schema + limits
packages/plugin-loader # validate/load zip & dir (no code execution)
packages/client-ui     # thin offline UI
tools/validator-cli    # CI-friendly pack validation
tools/bundle-mod.mjs   # packs reference-mod → public JSON
mods/reference-mod     # original sample content only
```

## Safety (MVP)

- Packs are **JSON data only** — no plugin script execution paths.
- Official content uses **original** names/assets only (no third-party IP catalogs).
