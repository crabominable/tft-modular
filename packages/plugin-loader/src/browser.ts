/**
 * Browser-safe entry: no node:fs / node:path.
 * Import from `@tft/plugin-loader/browser` in client-ui.
 */

export {
  API_VERSION,
  MAX_ASSET_BYTES,
  MAX_ABILITIES,
  MAX_PACK_BYTES,
  MAX_TRAITS,
  MAX_UNITS,
  ALLOWED_ASSET_EXTS,
  assertWithinLimits,
} from "./limits.ts";
export type { PackCounts } from "./limits.ts";

export { isSafePackPath, unzipPackBytes } from "./zip.ts";
export type { UnzipResult } from "./zip.ts";

export { hashCanonicalPack } from "./hash.ts";

export {
  validateManifestJson,
  validateUnitJson,
  validateTraitJson,
  validateAbilityJson,
} from "./validate.ts";
export type { ValidationResult } from "./validate.ts";

export { loadPackFromFiles, loadPackFromZip } from "./load-core.ts";
export type {
  AbilityDef,
  LoadResult,
  Manifest,
  PluginPack,
  TraitDef,
  UnitDef,
} from "./load-core.ts";
