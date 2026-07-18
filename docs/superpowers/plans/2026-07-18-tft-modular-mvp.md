# TFT Modular MVP Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship a browser-playable offline auto-battler that loads a data-only original reference mod, runs a lean shop→combat loop with deterministic replay, and keeps rules in a Rust core compiled to Wasm.

**Architecture:** Monorepo with `engine-core` (Rust, pure simulation), `engine-wasm` (wasm-bindgen glue), TypeScript `plugin-schema` / `plugin-loader` / `client-ui`, and `mods/reference-mod`. Loader validates and hashes packs; core only accepts in-memory `PluginData` and player `Command`s; UI is a thin command/event shell. No networking in MVP.

**Tech Stack:** Rust 2021, Cargo workspace, `serde`/`serde_json`, `wasm-bindgen`, `wasm-pack`; Node 20+, pnpm (or npm) workspaces; Vite + TypeScript; AJV for JSON Schema; `fflate` for zip in browser; SHA-256 via Web Crypto.

**Spec:** `docs/superpowers/specs/2026-07-18-tft-modular-mvp-design.md`

---

## File map (create during plan)

```text
tft_modular/
  Cargo.toml                          # workspace
  package.json                        # pnpm workspace root
  pnpm-workspace.yaml
  README.md
  .gitignore
  crates/
    engine-core/
      Cargo.toml
      src/
        lib.rs
        fixed.rs                      # i64 milli-units helpers
        rng.rs                        # seeded XorShift/Pcg
        ids.rs
        plugin.rs                     # PluginData types (serde)
        economy.rs
        pool.rs
        shop.rs
        combat/
          mod.rs
          targeting.rs
          effects.rs
        match_state.rs                # state machine
        command.rs
        replay.rs
        hash_state.rs
      tests/
        determinism.rs
        economy_tests.rs
        combat_tests.rs
    engine-wasm/
      Cargo.toml
      src/lib.rs                      # wasm exports
  packages/
    plugin-schema/
      package.json
      schemas/
        manifest.schema.json
        unit.schema.json
        trait.schema.json
        ability.schema.json
        pack.schema.json              # bundle doc if needed
      src/index.ts                    # re-export schema paths + constants
    plugin-loader/
      package.json
      src/
        index.ts
        limits.ts
        zip.ts
        hash.ts
        validate.ts
        load.ts
      src/limits.test.ts
      src/validate.test.ts
      src/zip.test.ts
    client-ui/
      package.json
      index.html
      vite.config.ts
      src/
        main.ts
        bridge.ts                     # load wasm + call API
        ui/
          app.ts
          shop.ts
          board.ts
          hud.ts
  mods/
    reference-mod/
      manifest.json
      units/
        ember_scout.json
        stone_warden.json
        mist_healer.json
      traits/
        emberkin.json
        bulwark.json
      abilities/
        spark_cut.json
        guard_pulse.json
        mist_balm.json
      assets/
        .gitkeep                      # optional placeholders later
  tools/
    validator-cli/
      package.json
      src/cli.ts
```

### Locked MVP parameters (implementation defaults)

| Parameter | Value |
|-----------|--------|
| `api_version` | `1.0.0` |
| Max pack bytes | `52_428_800` (50 MiB) |
| Max units / traits / abilities | `256` / `64` / `128` |
| Max asset file | `8_388_608` (8 MiB) |
| Max image edge | `2048` px (enforced later if decoding; MVP: reject by declared size optional) |
| Shop slots | `5` |
| Board | `4` columns × `2` rows per side (8 cells/side) |
| Bench | `9` slots |
| Players | `1` human + `1` AI |
| Starting gold | `3` then standard curve later; round income base `5` + interest |
| Interest | `min(5, gold / 10)` integer division |
| XP per buy | `4` gold → `4` XP; level thresholds: `2→2, 3→6, 4→10, 5→20, 6→36, 7→56, 8→80, 9→100` (cumulative XP to reach level) |
| Unit pool copies by cost | cost1:`30`, cost2:`25`, cost3:`18`, cost4:`10`, cost5:`9` |
| Combat tick | `50` ms sim units; max ticks `600` (30s) |
| Fixed-point | `1` unit = `1000` milli (`Fp = i64`) |
| RNG | `u64` seed, xorshift64* |

---

### Task 1: Bootstrap monorepo (git, Cargo, pnpm)

**Files:**
- Create: `.gitignore`, `README.md`, `Cargo.toml`, `package.json`, `pnpm-workspace.yaml`
- Create: `crates/engine-core/Cargo.toml`, `crates/engine-core/src/lib.rs`
- Create: `crates/engine-wasm/Cargo.toml`, `crates/engine-wasm/src/lib.rs` (stub)

- [ ] **Step 1: Initialize git**

```bash
cd "D:/[DESENVOLVIMENTO]/[WORKING]/tft_modular"
git init
```

- [ ] **Step 2: Write root `.gitignore`**

```gitignore
/target
**/node_modules
/dist
/packages/client-ui/dist
/crates/engine-wasm/pkg
*.log
.DS_Store
.env
.vscode
.idea
```

- [ ] **Step 3: Write workspace `Cargo.toml`**

```toml
[workspace]
resolver = "2"
members = [
  "crates/engine-core",
  "crates/engine-wasm",
]

[workspace.package]
edition = "2021"
license = "MIT"
version = "0.1.0"
```

- [ ] **Step 4: Write `crates/engine-core/Cargo.toml` and stub `lib.rs`**

```toml
[package]
name = "engine-core"
version.workspace = true
edition.workspace = true

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"

[dev-dependencies]
```

```rust
// crates/engine-core/src/lib.rs
#![deny(unsafe_code)]

pub fn engine_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_semver_like() {
        assert!(!engine_version().is_empty());
    }
}
```

- [ ] **Step 5: Write `crates/engine-wasm/Cargo.toml` stub (cdylib later)**

```toml
[package]
name = "engine-wasm"
version.workspace = true
edition.workspace = true

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
engine-core = { path = "../engine-core" }
wasm-bindgen = "0.2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde-wasm-bindgen = "0.6"

[dependencies.web-sys]
version = "0.3"
features = []
```

```rust
// crates/engine-wasm/src/lib.rs
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn wasm_engine_version() -> String {
    engine_core::engine_version().to_string()
}
```

- [ ] **Step 6: Write JS workspace roots**

```json
{
  "name": "tft-modular",
  "private": true,
  "packageManager": "pnpm@9.15.0",
  "scripts": {
    "test": "pnpm -r test",
    "build:wasm": "wasm-pack build crates/engine-wasm --target web --out-dir pkg",
    "validate-mod": "pnpm --filter validator-cli start -- ../../mods/reference-mod",
    "dev": "pnpm --filter client-ui dev"
  }
}
```

```yaml
# pnpm-workspace.yaml
packages:
  - "packages/*"
  - "tools/*"
```

- [ ] **Step 7: Write minimal `README.md`**

```markdown
# TFT Modular

Offline web auto-battler with data-only plugins and a Rust/Wasm rules core.

See `docs/superpowers/specs/2026-07-18-tft-modular-mvp-design.md`.
```

- [ ] **Step 8: Verify Rust tests**

Run: `cargo test -p engine-core`  
Expected: PASS (`version_is_semver_like`)

- [ ] **Step 9: Commit**

```bash
git add .gitignore README.md Cargo.toml package.json pnpm-workspace.yaml crates
git commit -m "chore: bootstrap cargo and js workspaces"
```

---

### Task 2: Plugin JSON Schema + constants package

**Files:**
- Create: `packages/plugin-schema/package.json`
- Create: `packages/plugin-schema/schemas/manifest.schema.json`
- Create: `packages/plugin-schema/schemas/unit.schema.json`
- Create: `packages/plugin-schema/schemas/trait.schema.json`
- Create: `packages/plugin-schema/schemas/ability.schema.json`
- Create: `packages/plugin-schema/src/index.ts`
- Create: `packages/plugin-schema/src/limits.ts`

- [ ] **Step 1: Package manifest**

```json
{
  "name": "@tft/plugin-schema",
  "version": "0.1.0",
  "private": true,
  "type": "module",
  "main": "src/index.ts",
  "exports": {
    ".": "./src/index.ts",
    "./schemas/*": "./schemas/*"
  }
}
```

- [ ] **Step 2: Write `limits.ts`**

```typescript
export const API_VERSION = "1.0.0";
export const MAX_PACK_BYTES = 52_428_800;
export const MAX_UNITS = 256;
export const MAX_TRAITS = 64;
export const MAX_ABILITIES = 128;
export const MAX_ASSET_BYTES = 8_388_608;
export const ALLOWED_ASSET_EXTS = [".png", ".webp", ".json"] as const;
```

- [ ] **Step 3: Write `manifest.schema.json`**

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://tft-modular.local/schemas/manifest.schema.json",
  "type": "object",
  "additionalProperties": false,
  "required": ["id", "name", "version", "api_version"],
  "properties": {
    "id": { "type": "string", "minLength": 1, "maxLength": 64, "pattern": "^[a-z0-9_\\-]+$" },
    "name": { "type": "string", "minLength": 1, "maxLength": 80 },
    "version": { "type": "string", "pattern": "^\\d+\\.\\d+\\.\\d+$" },
    "api_version": { "type": "string", "const": "1.0.0" },
    "description": { "type": "string", "maxLength": 500 }
  }
}
```

- [ ] **Step 4: Write `unit.schema.json`**

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://tft-modular.local/schemas/unit.schema.json",
  "type": "object",
  "additionalProperties": false,
  "required": ["id", "name", "cost", "traits", "stats"],
  "properties": {
    "id": { "type": "string", "minLength": 1, "maxLength": 64, "pattern": "^[a-z0-9_\\-]+$" },
    "name": { "type": "string", "minLength": 1, "maxLength": 80 },
    "cost": { "type": "integer", "minimum": 1, "maximum": 5 },
    "traits": {
      "type": "array",
      "items": { "type": "string" },
      "maxItems": 4,
      "uniqueItems": true
    },
    "ability_id": { "type": ["string", "null"] },
    "stats": {
      "type": "object",
      "additionalProperties": false,
      "required": ["hp", "atk", "range", "attack_speed_milli"],
      "properties": {
        "hp": { "type": "integer", "minimum": 1, "maximum": 100000 },
        "atk": { "type": "integer", "minimum": 0, "maximum": 100000 },
        "armor": { "type": "integer", "minimum": 0, "maximum": 1000, "default": 0 },
        "mr": { "type": "integer", "minimum": 0, "maximum": 1000, "default": 0 },
        "range": { "type": "integer", "minimum": 1, "maximum": 6 },
        "attack_speed_milli": { "type": "integer", "minimum": 100, "maximum": 5000 }
      }
    }
  }
}
```

- [ ] **Step 5: Write `trait.schema.json`**

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://tft-modular.local/schemas/trait.schema.json",
  "type": "object",
  "additionalProperties": false,
  "required": ["id", "name", "breakpoints"],
  "properties": {
    "id": { "type": "string", "pattern": "^[a-z0-9_\\-]+$" },
    "name": { "type": "string", "minLength": 1, "maxLength": 80 },
    "breakpoints": {
      "type": "array",
      "minItems": 1,
      "maxItems": 6,
      "items": {
        "type": "object",
        "additionalProperties": false,
        "required": ["min_units", "modifiers"],
        "properties": {
          "min_units": { "type": "integer", "minimum": 1, "maximum": 9 },
          "modifiers": {
            "type": "array",
            "maxItems": 8,
            "items": {
              "type": "object",
              "additionalProperties": false,
              "required": ["stat", "amount"],
              "properties": {
                "stat": { "type": "string", "enum": ["hp", "atk", "armor", "mr"] },
                "amount": { "type": "integer", "minimum": -10000, "maximum": 10000 }
              }
            }
          }
        }
      }
    }
  }
}
```

- [ ] **Step 6: Write `ability.schema.json`**

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://tft-modular.local/schemas/ability.schema.json",
  "type": "object",
  "additionalProperties": false,
  "required": ["id", "name", "trigger", "targeting", "effects"],
  "properties": {
    "id": { "type": "string", "pattern": "^[a-z0-9_\\-]+$" },
    "name": { "type": "string", "minLength": 1, "maxLength": 80 },
    "mana_cost": { "type": "integer", "minimum": 0, "maximum": 200, "default": 100 },
    "trigger": {
      "type": "string",
      "enum": ["ON_CAST", "ON_HIT", "ON_DEATH", "ROUND_START"]
    },
    "targeting": {
      "type": "object",
      "additionalProperties": false,
      "required": ["type"],
      "properties": {
        "type": {
          "type": "string",
          "enum": ["SELF", "NEAREST_ENEMY", "RANDOM_ENEMY", "ALL_ENEMIES"]
        },
        "count": { "type": "integer", "minimum": 1, "maximum": 12 }
      }
    },
    "effects": {
      "type": "array",
      "minItems": 1,
      "maxItems": 8,
      "items": {
        "type": "object",
        "additionalProperties": false,
        "required": ["type"],
        "properties": {
          "type": {
            "type": "string",
            "enum": ["DAMAGE", "HEAL", "SHIELD", "STUN", "STAT_MOD"]
          },
          "amount": { "type": "integer", "minimum": 0, "maximum": 100000 },
          "duration_ms": { "type": "integer", "minimum": 0, "maximum": 60000 },
          "stat": { "type": "string", "enum": ["hp", "atk", "armor", "mr"] },
          "scaling": { "type": "string", "enum": ["NONE", "AP", "ATK"] }
        }
      }
    }
  }
}
```

- [ ] **Step 7: Export package entry**

```typescript
// packages/plugin-schema/src/index.ts
export * from "./limits.js";
```

- [ ] **Step 8: Commit**

```bash
git add packages/plugin-schema
git commit -m "feat(schema): add plugin JSON schemas and limits"
```

---

### Task 3: Reference mod (original content only)

**Files:**
- Create: all files under `mods/reference-mod/` listed in file map

- [ ] **Step 1: Write `manifest.json`**

```json
{
  "id": "reference_embers",
  "name": "Ember Frontier",
  "version": "0.1.0",
  "api_version": "1.0.0",
  "description": "Original sample pack for TFT Modular offline MVP."
}
```

- [ ] **Step 2: Write three units, two traits, three abilities** (use exact ids from file map)

`units/ember_scout.json`:

```json
{
  "id": "ember_scout",
  "name": "Ember Scout",
  "cost": 1,
  "traits": ["emberkin"],
  "ability_id": "spark_cut",
  "stats": {
    "hp": 500,
    "atk": 45,
    "armor": 10,
    "mr": 10,
    "range": 1,
    "attack_speed_milli": 700
  }
}
```

`units/stone_warden.json`:

```json
{
  "id": "stone_warden",
  "name": "Stone Warden",
  "cost": 2,
  "traits": ["bulwark"],
  "ability_id": "guard_pulse",
  "stats": {
    "hp": 800,
    "atk": 40,
    "armor": 40,
    "mr": 20,
    "range": 1,
    "attack_speed_milli": 550
  }
}
```

`units/mist_healer.json`:

```json
{
  "id": "mist_healer",
  "name": "Mist Healer",
  "cost": 2,
  "traits": ["emberkin"],
  "ability_id": "mist_balm",
  "stats": {
    "hp": 450,
    "atk": 30,
    "armor": 15,
    "mr": 25,
    "range": 3,
    "attack_speed_milli": 650
  }
}
```

`traits/emberkin.json`:

```json
{
  "id": "emberkin",
  "name": "Emberkin",
  "breakpoints": [
    {
      "min_units": 2,
      "modifiers": [{ "stat": "atk", "amount": 15 }]
    }
  ]
}
```

`traits/bulwark.json`:

```json
{
  "id": "bulwark",
  "name": "Bulwark",
  "breakpoints": [
    {
      "min_units": 1,
      "modifiers": [{ "stat": "armor", "amount": 20 }]
    }
  ]
}
```

`abilities/spark_cut.json`:

```json
{
  "id": "spark_cut",
  "name": "Spark Cut",
  "mana_cost": 60,
  "trigger": "ON_CAST",
  "targeting": { "type": "NEAREST_ENEMY", "count": 1 },
  "effects": [
    { "type": "DAMAGE", "amount": 120, "scaling": "ATK" }
  ]
}
```

`abilities/guard_pulse.json`:

```json
{
  "id": "guard_pulse",
  "name": "Guard Pulse",
  "mana_cost": 80,
  "trigger": "ON_CAST",
  "targeting": { "type": "SELF" },
  "effects": [
    { "type": "SHIELD", "amount": 200, "duration_ms": 3000 }
  ]
}
```

`abilities/mist_balm.json`:

```json
{
  "id": "mist_balm",
  "name": "Mist Balm",
  "mana_cost": 70,
  "trigger": "ON_CAST",
  "targeting": { "type": "SELF" },
  "effects": [
    { "type": "HEAL", "amount": 180, "scaling": "NONE" }
  ]
}
```

- [ ] **Step 3: Commit**

```bash
git add mods/reference-mod
git commit -m "feat(mod): add original Ember Frontier reference pack"
```

---

### Task 4: Validator CLI + loader validation (Node)

**Files:**
- Create: `tools/validator-cli/package.json`, `tools/validator-cli/src/cli.ts`
- Create: `packages/plugin-loader/package.json` and sources listed in file map
- Test: `packages/plugin-loader/src/validate.test.ts`, `zip.test.ts`, `limits.test.ts`

- [ ] **Step 1: Implement shared validation in `plugin-loader`**

`packages/plugin-loader/package.json`:

```json
{
  "name": "@tft/plugin-loader",
  "version": "0.1.0",
  "private": true,
  "type": "module",
  "scripts": {
    "test": "node --experimental-strip-types --test src/**/*.test.ts"
  },
  "dependencies": {
    "@tft/plugin-schema": "workspace:*",
    "ajv": "^8.17.1",
    "ajv-formats": "^3.0.1",
    "fflate": "^0.8.2"
  }
}
```

Implement (full logic required in code when executing):

- `isSafePackPath(path: string): boolean` — reject `..`, absolute, backslash drive, empty segments.
- `assertWithinLimits(counts, totalBytes)`.
- `validateManifest/Unit/Trait/Ability` via AJV compiled schemas loaded from `@tft/plugin-schema/schemas/*`.
- `hashCanonicalPack(files: Map<string, Uint8Array>): Promise<string>` — sort paths UTF-8, SHA-256 over `path\0len\0bytes` concatenation using `crypto.subtle` (browser) or `node:crypto` (CLI).
- `loadPackFromDirectory(dir)` for Node CLI; `loadPackFromZip(bytes)` for browser.

- [ ] **Step 2: Write failing tests first**

```typescript
// packages/plugin-loader/src/limits.test.ts
import test from "node:test";
import assert from "node:assert/strict";
import { isSafePackPath } from "./zip.ts";

test("rejects parent traversal", () => {
  assert.equal(isSafePackPath("../secret.json"), false);
  assert.equal(isSafePackPath("units/../../x"), false);
});

test("accepts normal relative paths", () => {
  assert.equal(isSafePackPath("units/ember_scout.json"), true);
});
```

```typescript
// packages/plugin-loader/src/validate.test.ts
import test from "node:test";
import assert from "node:assert/strict";
import { validateUnitJson } from "./validate.ts";

test("valid unit passes", () => {
  const unit = {
    id: "ember_scout",
    name: "Ember Scout",
    cost: 1,
    traits: ["emberkin"],
    ability_id: "spark_cut",
    stats: { hp: 500, atk: 45, armor: 10, mr: 10, range: 1, attack_speed_milli: 700 },
  };
  assert.equal(validateUnitJson(unit).ok, true);
});

test("unknown ability effect type fails at pack link stage later; unit schema still ok without ability body", () => {
  const bad = { id: "x", name: "X", cost: 1, traits: [], stats: { hp: 1, atk: 1, range: 1, attack_speed_milli: 100 } };
  // missing nothing required — ok
  assert.equal(validateUnitJson(bad).ok, true);
});
```

- [ ] **Step 3: Run tests — expect FAIL (modules missing)**

Run: `pnpm install` then `pnpm --filter @tft/plugin-loader test`  
Expected: FAIL until implementation exists

- [ ] **Step 4: Implement `zip.ts`, `validate.ts`, `hash.ts`, `load.ts`, `index.ts`** until tests pass

Canonical hash sketch:

```typescript
export async function hashCanonicalPack(
  files: Map<string, Uint8Array>,
): Promise<string> {
  const paths = [...files.keys()].sort();
  const chunks: Uint8Array[] = [];
  const enc = new TextEncoder();
  for (const p of paths) {
    const data = files.get(p)!;
    chunks.push(enc.encode(p));
    chunks.push(enc.encode("\0"));
    chunks.push(enc.encode(String(data.byteLength)));
    chunks.push(enc.encode("\0"));
    chunks.push(data);
  }
  const total = concat(chunks);
  // node:
  const { createHash } = await import("node:crypto");
  return createHash("sha256").update(total).digest("hex");
}
```

Cross-link pass after individual schema validation:

- Every `traits[]` id exists.
- Every `ability_id` exists if non-null.
- Counts ≤ MAX_*.

- [ ] **Step 5: Validator CLI**

```typescript
// tools/validator-cli/src/cli.ts
import path from "node:path";
import { loadPackFromDirectory } from "@tft/plugin-loader";

const dir = process.argv[2];
if (!dir) {
  console.error("usage: validator-cli <mod-dir>");
  process.exit(2);
}
const result = await loadPackFromDirectory(path.resolve(dir));
if (!result.ok) {
  console.error(result.error);
  process.exit(1);
}
console.log(`OK ${result.pack.manifest.id} hash=${result.modHash}`);
```

`tools/validator-cli/package.json`:

```json
{
  "name": "validator-cli",
  "private": true,
  "type": "module",
  "scripts": {
    "start": "node --experimental-strip-types src/cli.ts"
  },
  "dependencies": {
    "@tft/plugin-loader": "workspace:*"
  }
}
```

- [ ] **Step 6: Run validator on reference mod**

Run: `pnpm install` ; `pnpm --filter validator-cli start -- ../../mods/reference-mod`  
Expected: `OK reference_embers hash=<64 hex chars>`

- [ ] **Step 7: Negative test fixture (inline in test)**

Create invalid pack in test memory with path `../x` or bad cost `9` → `ok === false`.

- [ ] **Step 8: Commit**

```bash
git add packages/plugin-loader tools/validator-cli pnpm-lock.yaml package.json
git commit -m "feat(loader): validate packs, hash canonically, add CLI"
```

---

### Task 5: engine-core foundations (fixed-point, RNG, plugin types)

**Files:**
- Create: `crates/engine-core/src/fixed.rs`, `rng.rs`, `ids.rs`, `plugin.rs`
- Modify: `crates/engine-core/src/lib.rs`
- Test: unit tests inside modules

- [ ] **Step 1: Write failing tests for RNG determinism and fixed mul**

```rust
// in rng.rs cfg(test)
#[test]
fn same_seed_same_sequence() {
    let mut a = Rng::new(0xDEAD_BEEF);
    let mut b = Rng::new(0xDEAD_BEEF);
    for _ in 0..100 {
        assert_eq!(a.next_u64(), b.next_u64());
    }
}

#[test]
fn different_seeds_diverge() {
    let mut a = Rng::new(1);
    let mut b = Rng::new(2);
    assert_ne!(a.next_u64(), b.next_u64());
}
```

- [ ] **Step 2: Implement `fixed.rs`**

```rust
//! Fixed-point helpers. 1.0 == 1000 milli.

pub type Fp = i64;
pub const FP_ONE: Fp = 1000;

#[inline]
pub fn fp_from_i64(v: i64) -> Fp {
    v.saturating_mul(FP_ONE)
}

#[inline]
pub fn fp_mul(a: Fp, b: Fp) -> Fp {
    // (a * b) / FP_ONE with i128 intermediate
    ((a as i128 * b as i128) / FP_ONE as i128) as i64
}
```

- [ ] **Step 3: Implement `rng.rs` (xorshift64*)**

```rust
#[derive(Clone, Debug)]
pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        // Avoid zero state
        Self { state: seed.rotate_left(1) ^ 0x9E37_79B9_7F4A_7C15 }
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    pub fn gen_range_usize(&mut self, max_exclusive: usize) -> usize {
        if max_exclusive == 0 {
            return 0;
        }
        (self.next_u64() as usize) % max_exclusive
    }
}
```

- [ ] **Step 4: Implement `plugin.rs` structs matching JSON** (`UnitDef`, `TraitDef`, `AbilityDef`, `PluginData`) with `serde::Deserialize`. Include `PluginData::from_json_files` helper used by tests (read strings, not FS in wasm).

- [ ] **Step 5: `cargo test -p engine-core` — PASS**

- [ ] **Step 6: Commit**

```bash
git add crates/engine-core
git commit -m "feat(core): fixed-point, rng, plugin data types"
```

---

### Task 6: Economy, pool, shop

**Files:**
- Create: `crates/engine-core/src/economy.rs`, `pool.rs`, `shop.rs`, `command.rs` (partial)
- Test: `crates/engine-core/tests/economy_tests.rs`

- [ ] **Step 1: Write failing integration tests**

```rust
// crates/engine-core/tests/economy_tests.rs
use engine_core::economy::{interest, level_from_xp, XP_THRESHOLDS};

#[test]
fn interest_caps_at_five() {
    assert_eq!(interest(0), 0);
    assert_eq!(interest(9), 0);
    assert_eq!(interest(10), 1);
    assert_eq!(interest(49), 4);
    assert_eq!(interest(50), 5);
    assert_eq!(interest(999), 5);
}

#[test]
fn level_thresholds_progress() {
    assert_eq!(level_from_xp(0), 1);
    assert_eq!(level_from_xp(2), 2);
    assert_eq!(level_from_xp(6), 3);
}
```

- [ ] **Step 2: Implement `economy.rs`** with constants from parameter table (`XP_THRESHOLDS`, `interest`, `level_from_xp`, `board_cap_for_level` = level).

- [ ] **Step 3: Implement `pool.rs`**

- Initialize bag counts per `unit_id` from cost tables × copies.
- `draw_shop(rng, level, shop_size=5)` uses level-weighted cost odds:

```text
level 1-2: c1=100
level 3: c1=75 c2=25
level 4: c1=55 c2=30 c3=15
level 5: c1=45 c2=33 c3=20 c4=2
level 6: c1=30 c2=40 c3=25 c4=5
level 7: c1=19 c2=30 c3=35 c4=15 c5=1
level 8: c1=18 c2=25 c3=32 c4=22 c5=3
level 9: c1=10 c2=20 c3=25 c4=35 c5=10
```

- Drawing removes one copy; buy removes from shop; sell returns 1 copy of base unit (star level 1 only in MVP — no stars yet).

- [ ] **Step 4: Implement shop actions on a `PlayerEconomy` + `ShopState`**

Commands for this task (unit-tested without full match):

- `reroll(cost=2)`
- `buy(shop_index)` if gold ≥ cost and bench not full
- `sell(unit_instance_id)` refund = cost (star1)

- [ ] **Step 5: `cargo test -p engine-core` — PASS**

- [ ] **Step 6: Commit**

```bash
git add crates/engine-core
git commit -m "feat(core): economy, unit pool, and shop actions"
```

---

### Task 7: Combat simulation (minimal)

**Files:**
- Create: `crates/engine-core/src/combat/mod.rs`, `targeting.rs`, `effects.rs`
- Test: `crates/engine-core/tests/combat_tests.rs`

- [ ] **Step 1: Failing test — nearest enemy takes damage until death**

```rust
#[test]
fn melee_kills_stationary_target() {
    // two units facing; attacker range 1; no movement needed if already adjacent
    // after enough ticks, defender hp <= 0 and combat result Winner SideA
}
```

- [ ] **Step 2: Implement combat state**

- Grid cells: Side A rows `y=0..1`, Side B mirrored `y=0..1` on opposite coordinate space OR single board with `y` 0–1 player and 2–3 enemy (choose **one board 4×4**: player occupies y=2..3, enemy y=0..1).
- Each combat unit: `hp`, `mana`, `shield`, `stun_until_tick`, `atk`, positions `(x,y)`.
- Each tick:
  1. Decrement timers.
  2. For each living unit in stable id order: if stunned skip; else acquire target; if in range attack or cast; else step one cell toward target (4-directional, deterministic tie-break: prefer smaller |dx| then axis x before y).
  3. Basic attack applies `atk` damage after armor: `damage = max(1, atk * 100 / (100 + armor))`.
  4. Mana +10 on attack; if mana ≥ cost and ability `ON_CAST`, fire effects, mana=0.
- End when one side all dead or `tick == MAX_TICKS` (draw → both lose round HP later; for 1v1 MVP deal 0 damage on draw).

- [ ] **Step 3: Effects allowlist only**

- `DAMAGE`: apply amount (+ optional ATK scaling: `amount + atk/2`)
- `HEAL`: add hp up to max
- `SHIELD`: set/add shield with duration ticks = duration_ms / 50
- `STUN`: set stun_until
- `STAT_MOD`: temporary atk/armor buff (store expiry)

- [ ] **Step 4: `cargo test -p engine-core --test combat_tests` — PASS**

- [ ] **Step 5: Commit**

```bash
git add crates/engine-core
git commit -m "feat(core): deterministic combat ticks and data abilities"
```

---

### Task 8: Match state machine + AI seat + replay hash

**Files:**
- Create: `crates/engine-core/src/match_state.rs`, `replay.rs`, `hash_state.rs`
- Modify: `command.rs`, `lib.rs`
- Test: `crates/engine-core/tests/determinism.rs`

- [ ] **Step 1: Define public API**

```rust
pub enum Command {
    BuyUnit { shop_index: u8 },
    SellUnit { unit_instance_id: u32 },
    Reroll,
    BuyExp,
    PlaceUnit { unit_instance_id: u32, cell: (u8, u8) },
    EndShopPhase,
}

pub struct Match {
    // plugin, rng, phase, players[2], round, pool, ...
}

impl Match {
    pub fn new(plugin: PluginData, seed: u64) -> Self { /* ... */ }
    pub fn apply(&mut self, player_id: u8, cmd: Command) -> Result<Vec<Event>, CoreError> { /* ... */ }
    pub fn phase(&self) -> Phase { /* ... */ }
    pub fn state_hash(&self) -> u64 { /* ... */ }
}
```

- [ ] **Step 2: Phase flow**

`Shop` → on both ready (human `EndShopPhase`, AI auto) → `Combat` (run full combat internally) → apply player damage (enemy board size surviving or fixed 2) → if player hp ≤ 0 `MatchEnd` else next `Shop` with income+interest.

Player HP start: `20`. Damage on loss: `2 + enemy_living_units`.

- [ ] **Step 3: AI shop (deterministic)**

On AI shop phase start and after human ends:

1. While gold ≥ 4 and level < 5: `BuyExp` (optional simple rule: if level < round+1 buy exp once).
2. Reroll at most once if no unit affordable.
3. Buy cheapest affordable slot left-to-right.
4. Place units on first empty board cells in row-major order up to board cap.
5. Auto `EndShopPhase`.

- [ ] **Step 4: Determinism test**

```rust
#[test]
fn identical_seeds_and_commands_match() {
    let plugin = load_reference_plugin_from_strings(); // include_str! files
    let cmds = vec![
        // sequence of human commands only; AI is pure function of state
        (0u8, Command::BuyUnit { shop_index: 0 }),
        (0, Command::EndShopPhase),
        // ...
    ];
    let h1 = run(plugin.clone(), 42, &cmds);
    let h2 = run(plugin, 42, &cmds);
    assert_eq!(h1, h2);
}
```

`state_hash`: stable FNV-1a or `sip` over gold, hp, unit ids/positions/hp, shop ids, phase, round — **not** memory addresses.

- [ ] **Step 5: `cargo test -p engine-core` — all PASS**

- [ ] **Step 6: Commit**

```bash
git add crates/engine-core
git commit -m "feat(core): match loop, AI seat, state hash, replay inputs"
```

---

### Task 9: engine-wasm JSON bridge

**Files:**
- Modify: `crates/engine-wasm/src/lib.rs`
- Create: glue types serializing `Command` / snapshots as JSON strings for simple JS interop

- [ ] **Step 1: Expose API**

```rust
#[wasm_bindgen]
pub struct WasmMatch {
    inner: engine_core::Match,
}

#[wasm_bindgen]
impl WasmMatch {
    #[wasm_bindgen(constructor)]
    pub fn new(plugin_json: &str, seed: u64) -> Result<WasmMatch, JsValue> {
        let plugin: engine_core::PluginData = serde_json::from_str(plugin_json)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(Self { inner: engine_core::Match::new(plugin, seed) })
    }

    pub fn apply(&mut self, player_id: u8, command_json: &str) -> Result<String, JsValue> {
        let cmd = serde_json::from_str(command_json)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let events = self.inner.apply(player_id, cmd)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        serde_json::to_string(&events).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    pub fn snapshot_json(&self) -> Result<String, JsValue> {
        serde_json::to_string(&self.inner.snapshot())
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    pub fn state_hash(&self) -> String {
        format!("{:016x}", self.inner.state_hash())
    }
}
```

`PluginData` for wasm path: single JSON object:

```json
{
  "manifest": { ... },
  "units": [ ... ],
  "traits": [ ... ],
  "abilities": [ ... ]
}
```

Loader must produce this shape; core deserializes it.

- [ ] **Step 2: Build wasm**

Run: `cargo install wasm-pack` (if needed); `wasm-pack build crates/engine-wasm --target web --out-dir ../../../packages/client-ui/src/wasm-pkg`  
Expected: `package` generated without error.

Note: put `pkg` under `packages/client-ui/src/wasm-pkg` and gitignore if huge; or `crates/engine-wasm/pkg` and depend via vite alias. Prefer **`packages/client-ui/public` not for wasm**; use:

```bash
wasm-pack build crates/engine-wasm --target web --out-dir ../../packages/client-ui/src/wasm-pkg
```

Add `packages/client-ui/src/wasm-pkg` to `.gitignore` **or** commit for convenience — prefer gitignore + build script.

- [ ] **Step 3: Commit glue source (not necessarily pkg)**

```bash
git add crates/engine-wasm Cargo.toml .gitignore
git commit -m "feat(wasm): expose Match JSON bridge to browser"
```

---

### Task 10: Thin client-ui vertical slice

**Files:**
- Create: `packages/client-ui/*` as in file map
- Wire: load reference mod via `@tft/plugin-loader` (directory fetch in dev using static copy), start match, shop buttons, board text/canvas, end phase

- [ ] **Step 1: Scaffold Vite app**

```json
{
  "name": "client-ui",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview"
  },
  "dependencies": {
    "@tft/plugin-loader": "workspace:*"
  },
  "devDependencies": {
    "typescript": "^5.7.0",
    "vite": "^6.0.0"
  }
}
```

Copy or statically serve `mods/reference-mod` from Vite:

```ts
// vite.config.ts
import { defineConfig } from "vite";
import path from "node:path";

export default defineConfig({
  server: {
    fs: { allow: ["../.."] },
  },
  resolve: {
    alias: {
      "@mod": path.resolve(__dirname, "../../mods/reference-mod"),
    },
  },
});
```

- [ ] **Step 2: `bridge.ts` loads wasm and constructs match**

```typescript
import init, { WasmMatch } from "./wasm-pkg/engine_wasm.js";

export async function createMatch(pluginBundle: unknown, seed: bigint | number) {
  await init();
  return new WasmMatch(JSON.stringify(pluginBundle), Number(seed));
}
```

- [ ] **Step 3: Minimal DOM UI**

- Show gold, level, HP, shop cards (name+cost), buttons: Buy 0..4, Reroll, Buy XP, End Shop.
- Board: list unit ids in cells (text grid is enough for MVP).
- Log last events JSON in a `<pre>`.
- On load: `fetch` each reference-mod JSON (or one bundled `bundle.json` generated by a small script). **Simplest path:** add `tools/bundle-mod.mjs` that writes `packages/client-ui/public/reference-mod.json` from the mod folder; client fetches `/reference-mod.json`.

Bundle script outline:

```js
// tools/bundle-mod.mjs
import fs from "node:fs";
import path from "node:path";
// read manifest, units/*, traits/*, abilities/* → write public/reference-mod.json
// also print hash via plugin-loader
```

- [ ] **Step 4: Manual test**

Run: `pnpm build:wasm` ; `node tools/bundle-mod.mjs` ; `pnpm --filter client-ui dev`  
Expected: browser opens; can buy unit; end shop; combat resolves; shop returns or match ends; `state_hash` displayed and stable on refresh+same seed+same clicks (document seed fixed `42` in UI for QA).

- [ ] **Step 5: Commit**

```bash
git add packages/client-ui tools/bundle-mod.mjs
git commit -m "feat(ui): offline vertical slice over wasm match"
```

---

### Task 11: Browser zip/URL loader path + local cache

**Files:**
- Modify: `packages/plugin-loader/src/load.ts`
- Modify: `packages/client-ui` to accept file input + optional URL

- [ ] **Step 1: Tests for zip path safety** using tiny zip fixtures built with `fflate` in test.

- [ ] **Step 2: Implement `loadPackFromZip(uint8)`** — decompress, safe paths, size limits, schema validate, hash, return `{ pluginBundle, modHash }`.

- [ ] **Step 3: Optional URL load in UI**

```typescript
async function loadFromUrl(url: string) {
  const res = await fetch(url);
  if (!res.ok) throw new Error(`fetch failed ${res.status}`);
  const buf = new Uint8Array(await res.arrayBuffer());
  if (buf.byteLength > MAX_PACK_BYTES) throw new Error("PackTooLarge");
  return loadPackFromZip(buf);
}
```

Cache: `indexedDB` key `mod:<hash>` store bundle JSON; on hit skip network.

- [ ] **Step 4: UI file input `<input type="file" accept=".zip">`**

- [ ] **Step 5: Commit**

```bash
git add packages/plugin-loader packages/client-ui
git commit -m "feat(loader): browser zip/url load with local hash cache"
```

---

### Task 12: Hardening, README, DoD checklist

**Files:**
- Modify: `README.md`
- Create: `docs/superpowers/plans/mvp-dod-checklist.md` (optional short)
- Add CI-friendly scripts to root `package.json` / `cargo test`

- [ ] **Step 1: Document run instructions in README**

```markdown
## Dev

Requirements: Rust, wasm-pack, Node 20+, pnpm

```bash
pnpm install
cargo test -p engine-core
pnpm --filter @tft/plugin-loader test
pnpm --filter validator-cli start -- ../../mods/reference-mod
pnpm build:wasm
node tools/bundle-mod.mjs
pnpm dev
```
```

- [ ] **Step 2: Run full verification matrix**

| Check | Command | Expected |
|-------|---------|----------|
| Core tests | `cargo test -p engine-core` | all pass |
| Loader tests | `pnpm --filter @tft/plugin-loader test` | all pass |
| Validator | `pnpm --filter validator-cli start -- ../../mods/reference-mod` | OK + hash |
| Determinism | included in core tests | pass |
| UI slice | manual | shop→combat→hash |

- [ ] **Step 3: Confirm DoD from spec**

- [ ] reference-mod validates
- [ ] invalid packs rejected
- [ ] deterministic replay/hash tests green
- [ ] browser playable offline
- [ ] no third-party IP strings in `mods/reference-mod` or docs examples
- [ ] no plugin code execution paths

- [ ] **Step 4: Final commit**

```bash
git add README.md
git commit -m "docs: MVP developer workflow and verification"
```

---

## Spec coverage self-review

| Spec requirement | Task(s) |
|------------------|---------|
| Web offline MVP | 10–11 |
| Lean TFT loop | 6–8 |
| Data-only plugins + schema | 2–4 |
| Rust core → Wasm | 5–9 |
| Deterministic replay/hash | 5, 8 |
| Safe loader + hash + limits | 4, 11 |
| Reference original mod | 3 |
| Validator CLI | 4 |
| No multiplayer | honored (no tasks) |
| No plugin scripts | schema allowlist only |
| Path to authoritative server | same `engine-core` API |

**Placeholder scan:** no TBD steps; numeric caps locked in parameter table.  
**Type consistency:** `Command`, `PluginData` bundle JSON, `mod_hash` hex, `state_hash` u64 hex string aligned across Tasks 8–10.

---

## Execution handoff

Plan complete and saved to `docs/superpowers/plans/2026-07-18-tft-modular-mvp.md`.

**Two execution options:**

1. **Subagent-Driven (recommended)** — fresh subagent per task, review between tasks, fast iteration (`subagent-driven-development`)
2. **Inline Execution** — execute tasks in this session with checkpoints (`executing-plans`)

Which approach?
