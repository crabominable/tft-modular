/**
 * Browser pack load: zip file, URL fetch, default JSON, local hash cache.
 * Network is only used for explicit URL load; cache never rehosts.
 */

import {
  loadPackFromZip,
  MAX_PACK_BYTES,
  type PluginPack,
} from "@tft/plugin-loader/browser";
import { getCachedMod, putCachedMod } from "./modCache.ts";
import type { PluginBundle } from "./types.ts";

export function packToBundle(pack: PluginPack, modHash: string): PluginBundle {
  return {
    manifest: pack.manifest,
    units: pack.units,
    traits: pack.traits,
    abilities: pack.abilities,
    modHash,
  };
}

export async function loadBundleFromZipBytes(
  bytes: Uint8Array,
): Promise<PluginBundle> {
  if (bytes.byteLength > MAX_PACK_BYTES) {
    throw new Error(`PackTooLarge: ${bytes.byteLength} > MAX_PACK_BYTES`);
  }
  const result = await loadPackFromZip(bytes);
  if (!result.ok) {
    throw new Error(result.error);
  }
  const bundle = packToBundle(result.pack, result.modHash);
  await putCachedMod(result.modHash, bundle);
  return bundle;
}

export async function loadBundleFromFile(file: File): Promise<PluginBundle> {
  if (file.size > MAX_PACK_BYTES) {
    throw new Error(`PackTooLarge: file ${file.size} > MAX_PACK_BYTES`);
  }
  const buf = new Uint8Array(await file.arrayBuffer());
  return loadBundleFromZipBytes(buf);
}

/**
 * Fetch a zip pack from URL. If `expectedHash` is set and present in
 * IndexedDB, skip network and return the cached bundle.
 */
export async function loadBundleFromUrl(
  url: string,
  expectedHash?: string,
): Promise<{ bundle: PluginBundle; fromCache: boolean }> {
  const hash = expectedHash?.trim();
  if (hash) {
    const hit = await getCachedMod(hash);
    if (hit) {
      return { bundle: hit, fromCache: true };
    }
  }

  const res = await fetch(url);
  if (!res.ok) {
    throw new Error(`fetch failed ${res.status}`);
  }
  const buf = new Uint8Array(await res.arrayBuffer());
  if (buf.byteLength > MAX_PACK_BYTES) {
    throw new Error("PackTooLarge");
  }

  const contentType = res.headers.get("content-type") ?? "";
  // Accept zip or opaque binary; JSON packs can still fall through zip fail.
  const looksJson =
    contentType.includes("application/json") ||
    url.toLowerCase().split("?")[0]!.endsWith(".json");

  if (looksJson) {
    const text = new TextDecoder().decode(buf);
    const raw = JSON.parse(text) as PluginBundle;
    if (!raw?.manifest?.id || !Array.isArray(raw.units)) {
      throw new Error("InvalidBundleJson");
    }
    if (raw.modHash) {
      await putCachedMod(raw.modHash, raw);
    }
    return { bundle: raw, fromCache: false };
  }

  const bundle = await loadBundleFromZipBytes(buf);
  if (hash && bundle.modHash && bundle.modHash.toLowerCase() !== hash.toLowerCase()) {
    throw new Error(
      `HashMismatch: expected ${hash}, got ${bundle.modHash}`,
    );
  }
  return { bundle, fromCache: false };
}

export async function loadBundleFromCache(
  hash: string,
): Promise<PluginBundle> {
  const hit = await getCachedMod(hash);
  if (!hit) {
    throw new Error(`CacheMiss: mod:${hash.trim()}`);
  }
  return hit;
}

/** Built-in offline slice: bundled reference-mod JSON. */
export async function loadDefaultReference(): Promise<PluginBundle> {
  const res = await fetch("/reference-mod.json");
  if (!res.ok) {
    throw new Error(
      `Failed to fetch /reference-mod.json (${res.status}). Run: node --experimental-strip-types tools/bundle-mod.mjs`,
    );
  }
  const bundle = (await res.json()) as PluginBundle;
  if (bundle.modHash) {
    await putCachedMod(bundle.modHash, bundle);
  }
  return bundle;
}
