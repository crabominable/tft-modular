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
 * Uses Web Crypto (browser + Node 20+).
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

  const subtle = globalThis.crypto?.subtle;
  if (!subtle) {
    throw new Error("Web Crypto SHA-256 is required (crypto.subtle unavailable)");
  }

  // Copy into a fresh ArrayBuffer to satisfy BufferSource typing across TS DOM libs.
  const ab = total.buffer.slice(
    total.byteOffset,
    total.byteOffset + total.byteLength,
  ) as ArrayBuffer;
  const digest = await subtle.digest("SHA-256", ab);
  return bytesToHex(new Uint8Array(digest));
}
