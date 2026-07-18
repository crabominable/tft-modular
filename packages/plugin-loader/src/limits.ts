import {
  MAX_ABILITIES,
  MAX_PACK_BYTES,
  MAX_TRAITS,
  MAX_UNITS,
} from "@tft/plugin-schema";

export {
  API_VERSION,
  MAX_ASSET_BYTES,
  MAX_ABILITIES,
  MAX_PACK_BYTES,
  MAX_TRAITS,
  MAX_UNITS,
  ALLOWED_ASSET_EXTS,
} from "@tft/plugin-schema";

export type PackCounts = {
  units: number;
  traits: number;
  abilities: number;
};

export function assertWithinLimits(counts: PackCounts, totalBytes: number): void {
  if (totalBytes > MAX_PACK_BYTES) {
    throw new Error(`PackTooLarge: ${totalBytes} > MAX_PACK_BYTES (${MAX_PACK_BYTES})`);
  }
  if (counts.units > MAX_UNITS) {
    throw new Error(`Too many units: ${counts.units} > MAX_UNITS (${MAX_UNITS})`);
  }
  if (counts.traits > MAX_TRAITS) {
    throw new Error(`Too many traits: ${counts.traits} > MAX_TRAITS (${MAX_TRAITS})`);
  }
  if (counts.abilities > MAX_ABILITIES) {
    throw new Error(
      `Too many abilities: ${counts.abilities} > MAX_ABILITIES (${MAX_ABILITIES})`,
    );
  }
}
