import { hashCanonicalPack } from "./hash.ts";
import { assertWithinLimits } from "./limits.ts";
import {
  validateAbilityJson,
  validateManifestJson,
  validateTraitJson,
  validateUnitJson,
} from "./validate.ts";
import { isSafePackPath, unzipPackBytes } from "./zip.ts";

export type Manifest = {
  id: string;
  name: string;
  version: string;
  api_version: string;
  description?: string;
};

export type UnitDef = {
  id: string;
  name: string;
  cost: number;
  traits: string[];
  ability_id?: string | null;
  stats: Record<string, number>;
};

export type TraitDef = {
  id: string;
  name: string;
  breakpoints: unknown[];
};

export type AbilityDef = {
  id: string;
  name: string;
  mana_cost?: number;
  trigger: string;
  targeting: unknown;
  effects: unknown[];
};

export type PluginPack = {
  manifest: Manifest;
  units: UnitDef[];
  traits: TraitDef[];
  abilities: AbilityDef[];
};

export type LoadResult =
  | { ok: true; pack: PluginPack; modHash: string; files: Map<string, Uint8Array> }
  | { ok: false; error: string };

function decodeJson(_pathKey: string, bytes: Uint8Array): unknown {
  const text = new TextDecoder("utf-8", { fatal: true }).decode(bytes);
  return JSON.parse(text);
}

function collectByPrefix(
  files: Map<string, Uint8Array>,
  prefix: string,
): [string, Uint8Array][] {
  const out: [string, Uint8Array][] = [];
  for (const [p, data] of files) {
    if (p.startsWith(prefix) && p.endsWith(".json") && !p.slice(prefix.length).includes("/")) {
      out.push([p, data]);
    }
  }
  return out.sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0));
}

function crossLink(
  units: UnitDef[],
  traits: TraitDef[],
  abilities: AbilityDef[],
): string | null {
  const traitIds = new Set(traits.map((t) => t.id));
  const abilityIds = new Set(abilities.map((a) => a.id));

  for (const unit of units) {
    for (const tid of unit.traits) {
      if (!traitIds.has(tid)) {
        return `UnknownTrait: unit ${unit.id} references missing trait '${tid}'`;
      }
    }
    if (unit.ability_id != null && unit.ability_id !== "") {
      if (!abilityIds.has(unit.ability_id)) {
        return `UnknownAbility: unit ${unit.id} references missing ability '${unit.ability_id}'`;
      }
    }
  }
  return null;
}

/**
 * Assemble, schema-validate, cross-link, and hash a pack from an in-memory file map.
 */
export async function loadPackFromFiles(
  files: Map<string, Uint8Array>,
): Promise<LoadResult> {
  let totalBytes = 0;
  for (const [p, data] of files) {
    if (!isSafePackPath(p)) {
      return { ok: false, error: `UnsafePackPath: ${p}` };
    }
    totalBytes += data.byteLength;
  }

  if (!files.has("manifest.json")) {
    return { ok: false, error: "Missing manifest.json" };
  }

  let manifestRaw: unknown;
  try {
    manifestRaw = decodeJson("manifest.json", files.get("manifest.json")!);
  } catch (err) {
    return {
      ok: false,
      error: `InvalidJson manifest.json: ${err instanceof Error ? err.message : String(err)}`,
    };
  }

  const manifestResult = validateManifestJson(manifestRaw);
  if (!manifestResult.ok) {
    return {
      ok: false,
      error: `InvalidManifest: ${manifestResult.errors.join("; ")}`,
    };
  }
  const manifest = manifestResult.value as Manifest;

  const units: UnitDef[] = [];
  const traits: TraitDef[] = [];
  const abilities: AbilityDef[] = [];
  const seenUnitIds = new Set<string>();
  const seenTraitIds = new Set<string>();
  const seenAbilityIds = new Set<string>();

  for (const [filePath, data] of collectByPrefix(files, "units/")) {
    let raw: unknown;
    try {
      raw = decodeJson(filePath, data);
    } catch (err) {
      return {
        ok: false,
        error: `InvalidJson ${filePath}: ${err instanceof Error ? err.message : String(err)}`,
      };
    }
    const result = validateUnitJson(raw);
    if (!result.ok) {
      return {
        ok: false,
        error: `InvalidUnit ${filePath}: ${result.errors.join("; ")}`,
      };
    }
    const unit = result.value as UnitDef;
    if (seenUnitIds.has(unit.id)) {
      return { ok: false, error: `DuplicateUnitId: ${unit.id}` };
    }
    seenUnitIds.add(unit.id);
    units.push(unit);
  }

  for (const [filePath, data] of collectByPrefix(files, "traits/")) {
    let raw: unknown;
    try {
      raw = decodeJson(filePath, data);
    } catch (err) {
      return {
        ok: false,
        error: `InvalidJson ${filePath}: ${err instanceof Error ? err.message : String(err)}`,
      };
    }
    const result = validateTraitJson(raw);
    if (!result.ok) {
      return {
        ok: false,
        error: `InvalidTrait ${filePath}: ${result.errors.join("; ")}`,
      };
    }
    const trait = result.value as TraitDef;
    if (seenTraitIds.has(trait.id)) {
      return { ok: false, error: `DuplicateTraitId: ${trait.id}` };
    }
    seenTraitIds.add(trait.id);
    traits.push(trait);
  }

  for (const [filePath, data] of collectByPrefix(files, "abilities/")) {
    let raw: unknown;
    try {
      raw = decodeJson(filePath, data);
    } catch (err) {
      return {
        ok: false,
        error: `InvalidJson ${filePath}: ${err instanceof Error ? err.message : String(err)}`,
      };
    }
    const result = validateAbilityJson(raw);
    if (!result.ok) {
      return {
        ok: false,
        error: `InvalidAbility ${filePath}: ${result.errors.join("; ")}`,
      };
    }
    const ability = result.value as AbilityDef;
    if (seenAbilityIds.has(ability.id)) {
      return { ok: false, error: `DuplicateAbilityId: ${ability.id}` };
    }
    seenAbilityIds.add(ability.id);
    abilities.push(ability);
  }

  try {
    assertWithinLimits(
      { units: units.length, traits: traits.length, abilities: abilities.length },
      totalBytes,
    );
  } catch (err) {
    return {
      ok: false,
      error: err instanceof Error ? err.message : String(err),
    };
  }

  const linkError = crossLink(units, traits, abilities);
  if (linkError) {
    return { ok: false, error: linkError };
  }

  const modHash = await hashCanonicalPack(files);
  return {
    ok: true,
    pack: { manifest, units, traits, abilities },
    modHash,
    files,
  };
}

/**
 * Load a pack from zip bytes (browser / CLI).
 */
export async function loadPackFromZip(bytes: Uint8Array): Promise<LoadResult> {
  const unzipped = unzipPackBytes(bytes);
  if (!unzipped.ok) {
    return { ok: false, error: unzipped.error };
  }
  return loadPackFromFiles(unzipped.files);
}
