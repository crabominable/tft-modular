function concat(chunks: Uint8Array[]): Uint8Array {
  let len = 0;
  for (const c of chunks) len += c.byteLength;
  const out = new Uint8Array(len);
  let offset = 0;
  for (const c of chunks) {
    out.set(c, offset);
    offset += c.byteLength;
  }
  return out;
}

function bytesToHex(bytes: Uint8Array): string {
  return Array.from(bytes)
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
}

/**
 * Canonical pack hash: sort paths UTF-8, SHA-256 over path\0len\0bytes concatenation.
 */
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

  // Prefer Web Crypto when available (browser / modern Node)
  if (globalThis.crypto?.subtle) {
    const digest = await globalThis.crypto.subtle.digest("SHA-256", total);
    return bytesToHex(new Uint8Array(digest));
  }

  const { createHash } = await import("node:crypto");
  return createHash("sha256").update(total).digest("hex");
}
