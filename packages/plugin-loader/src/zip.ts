import { unzipSync } from "fflate";
import { MAX_PACK_BYTES } from "./limits.ts";

/**
 * Reject path traversal, absolute paths, drive letters, backslashes, empty segments.
 */
export function isSafePackPath(path: string): boolean {
  if (typeof path !== "string" || path.length === 0) return false;
  if (path.includes("\\")) return false;
  if (path.startsWith("/")) return false;
  if (/^[a-zA-Z]:/.test(path)) return false;
  if (path.includes("\0")) return false;

  const segments = path.split("/");
  for (const seg of segments) {
    if (seg === "" || seg === "." || seg === "..") return false;
  }
  return true;
}

export type UnzipResult =
  | { ok: true; files: Map<string, Uint8Array>; totalBytes: number }
  | { ok: false; error: string };

/**
 * Decompress zip bytes into a path → bytes map with safety checks.
 */
export function unzipPackBytes(bytes: Uint8Array): UnzipResult {
  if (bytes.byteLength > MAX_PACK_BYTES) {
    return {
      ok: false,
      error: `PackTooLarge: compressed size ${bytes.byteLength} > MAX_PACK_BYTES`,
    };
  }

  let raw: Record<string, Uint8Array>;
  try {
    raw = unzipSync(bytes);
  } catch (err) {
    return {
      ok: false,
      error: `InvalidZip: ${err instanceof Error ? err.message : String(err)}`,
    };
  }

  const files = new Map<string, Uint8Array>();
  let totalBytes = 0;

  for (const [rawPath, data] of Object.entries(raw)) {
    if (rawPath.endsWith("/")) continue;

    let path = rawPath.replace(/\\/g, "/");
    while (path.startsWith("./")) path = path.slice(2);

    if (!isSafePackPath(path)) {
      return { ok: false, error: `UnsafePackPath: ${rawPath}` };
    }

    totalBytes += data.byteLength;
    if (totalBytes > MAX_PACK_BYTES) {
      return {
        ok: false,
        error: `PackTooLarge: uncompressed ${totalBytes} > MAX_PACK_BYTES`,
      };
    }

    if (files.has(path)) {
      return { ok: false, error: `DuplicatePath: ${path}` };
    }
    files.set(path, data);
  }

  return { ok: true, files, totalBytes };
}
