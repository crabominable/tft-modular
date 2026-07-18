import { readdir, readFile, stat } from "node:fs/promises";
import path from "node:path";
import { isSafePackPath } from "./zip.ts";
import { loadPackFromFiles, type LoadResult } from "./load-core.ts";

async function walkDirectory(
  root: string,
  base: string,
  out: Map<string, Uint8Array>,
): Promise<string | null> {
  const entries = await readdir(base, { withFileTypes: true });
  for (const entry of entries) {
    const abs = path.join(base, entry.name);
    if (entry.isDirectory()) {
      const err = await walkDirectory(root, abs, out);
      if (err) return err;
      continue;
    }
    if (!entry.isFile()) continue;

    const rel = path.relative(root, abs).split(path.sep).join("/");
    if (!isSafePackPath(rel)) {
      return `UnsafePackPath: ${rel}`;
    }
    const buf = await readFile(abs);
    out.set(rel, new Uint8Array(buf));
  }
  return null;
}

/**
 * Load a pack from a filesystem directory (Node CLI).
 */
export async function loadPackFromDirectory(dir: string): Promise<LoadResult> {
  const resolved = path.resolve(dir);
  let st;
  try {
    st = await stat(resolved);
  } catch {
    return { ok: false, error: `DirectoryNotFound: ${resolved}` };
  }
  if (!st.isDirectory()) {
    return { ok: false, error: `NotADirectory: ${resolved}` };
  }

  const files = new Map<string, Uint8Array>();
  const walkErr = await walkDirectory(resolved, resolved, files);
  if (walkErr) {
    return { ok: false, error: walkErr };
  }

  return loadPackFromFiles(files);
}
