# TFT Modular — MVP Design Spec

**Date:** 2026-07-18  
**Status:** Approved for implementation planning  
**Notion hub:** [TFT modular](https://app.notion.com/p/TFT-modular-8774462297c1408fa694662ecc15258b)  
**Related planning:** product phases, legal/security threats, and multiplayer roadmap live in Notion; this doc is the **MVP technical design** only.

---

## 1. Goal

Build a **web-based auto-battler** with a **content-agnostic engine** and **data-only plugins** (mods). The MVP proves:

1. A versioned plugin contract (schema + validator).
2. Offline match loop: economy → shop → combat rounds → match end.
3. Deterministic simulation with replay.
4. Safe local plugin load (URL or zip) with hashing and hard limits.
5. A clean path to authoritative multiplayer later **without rewriting the rules core**.

---

## 2. Locked product decisions (Phase 0)

| Decision | Choice |
|----------|--------|
| Platform | **Web** (browser) |
| Networking | **Offline only** in MVP |
| Playable surface | **Lean TFT loop** (economy, shared pool/shop, buy/sell, reroll, exp, interest, multi-round combat) |
| Plugin model | **Data only** (JSON + assets). No plugin scripts. |
| Architecture | **Rust simulation core → Wasm** + TypeScript UI/loader shell |
| Official content | **Original reference mod only** (no third-party IP in repo/docs/examples) |
| Multiplayer | **Out of scope** (Phase 3); same core intended for headless server later |

### Explicit non-goals (MVP)

- Matchmaking, authoritative server, ranked
- Complex items, augments, carousel, full 8-player polish beyond minimal AI seats
- Plugin scripts / general-purpose DSL / plugin Wasm
- Official public mod catalog or CDN rehosting of third-party packs
- Monetization, accounts, mobile-first polish

---

## 3. Architecture

```
┌─────────────────────────────────────────────────────────┐
│  client-ui (TypeScript + Vite)                          │
│  HUD, shop, board presentation                          │
│  Sends Commands; renders Events / Snapshots only        │
└──────────────────────────┬──────────────────────────────┘
                           │ JS ↔ Wasm bridge
┌──────────────────────────▼──────────────────────────────┐
│  engine-core (Rust → Wasm)                              │
│  Match state machine, economy, pool, shop, combat       │
│  Fixed-point math, seeded RNG, deterministic resolve    │
│  Consumes in-memory PluginData (no network/FS I/O)      │
└──────────────────────────▲──────────────────────────────┘
                           │ validated PluginData
┌──────────────────────────┴──────────────────────────────┐
│  plugin-loader (TypeScript, browser)                    │
│  URL fetch or zip upload; path/MIME/size limits         │
│  JSON Schema validation; content hash; local cache only │
└─────────────────────────────────────────────────────────┘
                           │
                           ▼
              plugin pack + mods/reference-mod
```

### Principles

1. **Rules never trust the client view.** Canonical state lives in `engine-core`.
2. **Core never loads untrusted bytes from the network.** Loader does I/O and validation; core receives structured `PluginData`.
3. **No code execution from packs** in MVP.
4. **No rehosting.** Cache is device-local only (e.g. Cache API / IndexedDB keyed by content hash).
5. **Same core later on server.** Phase 3 runs `engine-core` headless; client remains thin.

---

## 4. Components

| Component | Responsibility | Must not |
|-----------|----------------|----------|
| `engine-core` | Deterministic match simulation | Network, filesystem, rendering |
| `engine-wasm` | Bindings / glue for browser | Game rules duplication |
| `plugin-schema` | Versioned JSON Schema + docs | Runtime side effects |
| `plugin-loader` | Ingest, limits, hash, schema validate, local cache | Decide combat/economy outcomes |
| `client-ui` | Input + presentation | Authoritative rule computation |
| `validator-cli` | CI/local pack validation | Serve the game |
| `mods/reference-mod` | Original sample content | Third-party IP names/assets |

### Bridge contract (thin)

**Commands (UI → core)**

- `StartMatch { mod_hash, seed }`
- `BuyUnit { shop_index }`
- `SellUnit { board_unit_id }`
- `Reroll`
- `BuyExp`
- `PlaceUnit { board_unit_id, cell }`
- `EndShopPhase` (or equivalent ready/lock)

Combat is auto-resolved; no continuous player combat input in MVP.

**Outputs (core → UI)**

- Phase / shop / board / pool snapshots
- Combat tick summaries or discrete combat events
- Round and match results
- Typed errors (`InvalidCommand`, `WrongPhase`, etc.)

UI may interpolate visuals; **never** invent gold, damage, or shop contents client-side.

---

## 5. Plugin contract (data-only)

### Layout

```text
my-mod/
  manifest.json
  units/*.json
  traits/*.json
  abilities/*.json
  assets/...
```

Assets may be local files under `assets/` or external URLs listed in manifest with **immutable pinning** (hash). Mutable “latest” URLs are rejected.

### `manifest.json` (required concepts)

- `id` (stable string)
- `name`, `version` (semver)
- `api_version` (engine plugin API major/minor)
- File inventory or conventional tree
- Optional declared limits (engine enforces max of declared vs global caps)

### Entity sketches

**Unit:** `id`, cost, trait ids, optional ability id, integer/fixed stats (hp, atk, armor, mr, range, attack_speed_fp, …).

**Trait:** `id`, stack thresholds → data-driven modifiers.

**Ability:** declarative composition only:

- Triggers (MVP): `ON_CAST`, `ON_HIT`, `ON_DEATH`, `ROUND_START`
- Targeting (MVP): `SELF`, `NEAREST_ENEMY`, `RANDOM_ENEMY`, `ALL_ENEMIES` (subject to balance caps)
- Effects (MVP allowlist): `DAMAGE`, `HEAL`, `SHIELD`, `STUN`, `STAT_MOD`

Unknown effect/trigger/target types → **load failure** (fail closed).

### Versioning

- Engine advertises supported `api_version` range.
- Incompatible major → reject pack before match start.

### Explicitly forbidden in packs

- JavaScript, Lua, Wasm modules, shell, macros that execute
- Arbitrary expressions beyond the closed effect schema
- Path escape, absolute paths, symlinks in zip

---

## 6. Match loop (lean TFT)

### State machine

```text
LoadMod → InitMatch(seed)
  → loop: ShopPhase → CombatPhase → ResolveRound
  → MatchEnd
```

### Players

- **1 human** seat controlled via Commands.
- **Additional seats filled by deterministic AI** (seeded heuristics: e.g. buy highest affordable, place in stable slot order).  
  MVP target: enough seats to exercise pool pressure (implementation plan may start **1 human + 1 AI** then scale toward 8 without schema changes).

### Shop phase

- Gold, level, XP, interest (single documented formula).
- Shop slots drawn from a shared pool model.
- Buy, sell, reroll, buy XP, place units on board/bench with capacity rules.

### Combat phase

- Fixed simulation timestep (integer time units).
- Grid movement + simple targeting.
- Abilities fire from data definitions.
- End condition: team wipe or tick limit (documented tie rule).

### Determinism requirements

- Seeded RNG with explicit streams (no JS `Math.random` in rules).
- Fixed-point or integer stats/positions (e.g. milli-units).
- Total order for simultaneous events (stable ids / slot indices).
- Replay = `(mod_hash, api_version, seed, command_log[])` → identical `final_state_hash`.

---

## 7. Loader, hashing, security

| Control | Rule |
|---------|------|
| Pack size | Global cap (concrete number in implementation plan; order of tens of MB) |
| Counts | Max units, traits, abilities, assets, shop definitions |
| Paths | Normalize; reject `..`, absolute paths, symlink escapes |
| Asset types | MIME/extension allowlist; max width/height/decoded bytes |
| Schema | Validate all JSON; fail closed |
| Hash | SHA-256 of canonical pack serialization (stable file order) |
| Cache | Local only, keyed by hash |
| Remote load | User-supplied URL; no platform CDN republish |
| Server SSRF | N/A in offline MVP; when Phase 3 arrives, server must not fetch arbitrary URLs without allowlist/hash/size policy |

### Cosmetics vs rules

- Missing cosmetic asset → placeholder, match may continue.
- Missing/invalid rules JSON → hard fail, no match.

### Typed load errors (illustrative)

`PackTooLarge`, `UnsafePath`, `InvalidSchema`, `UnsupportedApiVersion`, `UnsupportedAssetType`, `HashMismatch`, `EntryLimitExceeded`.

---

## 8. Data flow (runtime)

1. User selects pack (URL or file/zip).
2. `plugin-loader` downloads/reads, enforces limits, validates schema, computes `mod_hash`, caches locally.
3. UI calls `StartMatch { mod_hash, seed }` with `PluginData` already resident (or core receives serialized validated blob once).
4. Core initializes match; UI renders snapshots.
5. Each player action is a Command; core advances and emits events.
6. On `MatchEnd`, UI can export replay.
7. Replay tool/tests re-run commands and assert `final_state_hash`.

---

## 9. Repository layout

```text
tft_modular/
  docs/superpowers/specs/
  crates/
    engine-core/
    engine-wasm/
  packages/
    client-ui/
    plugin-loader/
    plugin-schema/
  mods/
    reference-mod/
  tools/
    validator-cli/
```

UI stack default for planning: **Vite + TypeScript**; board via Canvas or a lightweight renderer; HUD in DOM. Exact UI library is an implementation detail and must not leak into `engine-core`.

---

## 10. Testing strategy

| Layer | What |
|-------|------|
| Schema | Valid reference mod; corpus of invalid packs rejected |
| Core unit | Economy formulas, shop draws, damage/stun/targeting |
| Determinism | Same seed+commands twice → same hashes; cross-run stability |
| Loader | Zip bomb, path traversal, oversize, bad MIME |
| Vertical slice | Browser: load reference mod → play short match → export/replay |

### MVP definition of done

A newcomer can load `reference-mod` in the browser, play a short offline match through shop and combat rounds, and a replay reproduces the same final state hash. Validator CLI passes in CI. No third-party IP strings in official content or docs.

---

## 11. Build order (for implementation plan)

1. `plugin-schema` + `mods/reference-mod` + `validator-cli`
2. `engine-core`: match state machine, economy, shop/pool, minimal combat
3. Determinism + replay tests in Rust
4. `engine-wasm` bridge + thin `client-ui`
5. Browser `plugin-loader` (file + URL)
6. Vertical slice polish + limit hardening

---

## 12. Legal / risk posture (MVP scope)

Inherited from Notion threat model; MVP implements:

- Neutral branding and original reference content only.
- No official third-party IP catalog.
- No rehosting of remote packs.
- Fail-closed data packs (no script execution).
- Treat TOS/community policy as later product work if discovery exists; not required for offline MVP.

Web platform does **not** weaken these controls if the architecture above is respected. Client-side cheats are irrelevant for offline single-player integrity; competitive integrity is deferred to Phase 3 authoritative server.

---

## 13. Open items deferred to implementation plan (not design blockers)

- Exact numeric caps (MB, max units, board size, round count).
- Exact interest/XP/level tables (must be documented once chosen).
- 2 vs 8 AI seats for first playable slice.
- Canvas vs Pixi (or similar) for board rendering.
- Canonical pack hashing algorithm details (file order, normalization).

These are **parameter choices**, not architectural forks.

---

## 14. Next process step

After user review of this file, create an implementation plan via the writing-plans skill:

`docs/superpowers/plans/2026-07-18-tft-modular-mvp.md` (path may follow writing-plans conventions).

Do **not** start feature implementation until that plan exists and is accepted.
