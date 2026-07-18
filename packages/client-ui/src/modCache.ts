/**
 * Device-local IndexedDB cache for validated mod bundles.
 * Keys: `mod:<content-hash>`. Never rehosts remotely.
 */

import type { PluginBundle } from "./types.ts";

const DB_NAME = "tft-mod-cache";
const DB_VERSION = 1;
const STORE = "mods";

function cacheKey(hash: string): string {
  const h = hash.trim().toLowerCase();
  return h.startsWith("mod:") ? h : `mod:${h}`;
}

function openDb(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const req = indexedDB.open(DB_NAME, DB_VERSION);
    req.onerror = () => reject(req.error ?? new Error("indexedDB open failed"));
    req.onsuccess = () => resolve(req.result);
    req.onupgradeneeded = () => {
      const db = req.result;
      if (!db.objectStoreNames.contains(STORE)) {
        db.createObjectStore(STORE);
      }
    };
  });
}

export async function getCachedMod(
  hash: string,
): Promise<PluginBundle | null> {
  if (!hash?.trim()) return null;
  const key = cacheKey(hash);
  const db = await openDb();
  try {
    return await new Promise((resolve, reject) => {
      const tx = db.transaction(STORE, "readonly");
      const store = tx.objectStore(STORE);
      const req = store.get(key);
      req.onerror = () => reject(req.error ?? new Error("cache get failed"));
      req.onsuccess = () => {
        const val = req.result;
        if (val && typeof val === "object" && "manifest" in val) {
          resolve(val as PluginBundle);
        } else {
          resolve(null);
        }
      };
    });
  } finally {
    db.close();
  }
}

export async function putCachedMod(
  hash: string,
  bundle: PluginBundle,
): Promise<void> {
  const key = cacheKey(hash);
  const db = await openDb();
  try {
    await new Promise<void>((resolve, reject) => {
      const tx = db.transaction(STORE, "readwrite");
      const store = tx.objectStore(STORE);
      const req = store.put({ ...bundle, modHash: hash.replace(/^mod:/i, "") }, key);
      req.onerror = () => reject(req.error ?? new Error("cache put failed"));
      req.onsuccess = () => resolve();
    });
  } finally {
    db.close();
  }
}
