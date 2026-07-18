import test from "node:test";
import assert from "node:assert/strict";
import { zipSync } from "fflate";
import { MAX_PACK_BYTES } from "./limits.ts";
import { isSafePackPath, unzipPackBytes } from "./zip.ts";
import { loadPackFromZip } from "./load.ts";

const enc = new TextEncoder();

function minimalManifest(id = "mini") {
  return {
    id,
    name: "Mini",
    version: "0.1.0",
    api_version: "1.0.0",
  };
}

function minimalValidZipFiles(id = "mini"): Record<string, Uint8Array> {
  return {
    "manifest.json": enc.encode(JSON.stringify(minimalManifest(id))),
    "units/u.json": enc.encode(
      JSON.stringify({
        id: "u",
        name: "U",
        cost: 1,
        traits: ["t"],
        stats: { hp: 100, atk: 10, range: 1, attack_speed_milli: 500 },
      }),
    ),
    "traits/t.json": enc.encode(
      JSON.stringify({
        id: "t",
        name: "T",
        breakpoints: [{ min_units: 1, modifiers: [{ stat: "atk", amount: 1 }] }],
      }),
    ),
  };
}

test("rejects parent traversal in zip paths", () => {
  assert.equal(isSafePackPath("../x"), false);
});

test("unzipPackBytes rejects unsafe paths", () => {
  const zipped = zipSync({
    "../secret.json": enc.encode("{}"),
  });
  const result = unzipPackBytes(zipped);
  assert.equal(result.ok, false);
  if (!result.ok) assert.match(result.error, /UnsafePackPath/);
});

test("unzipPackBytes rejects absolute path entries", () => {
  const zipped = zipSync({
    "/etc/passwd": enc.encode("nope"),
  });
  const result = unzipPackBytes(zipped);
  assert.equal(result.ok, false);
  if (!result.ok) assert.match(result.error, /UnsafePackPath/);
});

test("unzipPackBytes rejects oversize compressed input", () => {
  const huge = new Uint8Array(MAX_PACK_BYTES + 1);
  const result = unzipPackBytes(huge);
  assert.equal(result.ok, false);
  if (!result.ok) assert.match(result.error, /PackTooLarge/);
});

test("unzipPackBytes accepts safe nested files and reports size", () => {
  const zipped = zipSync(minimalValidZipFiles());
  const result = unzipPackBytes(zipped);
  assert.equal(result.ok, true);
  if (result.ok) {
    assert.equal(result.files.has("manifest.json"), true);
    assert.equal(result.files.has("units/u.json"), true);
    assert.ok(result.totalBytes > 0);
    assert.ok(result.totalBytes <= MAX_PACK_BYTES);
  }
});

test("loadPackFromZip rejects pack with unsafe path", async () => {
  const zipped = zipSync({
    "manifest.json": enc.encode(JSON.stringify(minimalManifest("z"))),
    "../x": enc.encode("{}"),
  });
  const result = await loadPackFromZip(zipped);
  assert.equal(result.ok, false);
  if (!result.ok) assert.match(result.error, /UnsafePackPath/);
});

test("loadPackFromZip rejects missing manifest", async () => {
  const zipped = zipSync({
    "units/u.json": enc.encode("{}"),
  });
  const result = await loadPackFromZip(zipped);
  assert.equal(result.ok, false);
  if (!result.ok) assert.match(result.error, /manifest/i);
});

test("loadPackFromZip accepts minimal valid pack", async () => {
  const zipped = zipSync(minimalValidZipFiles("mini"));
  const result = await loadPackFromZip(zipped);
  assert.equal(result.ok, true);
  if (result.ok) {
    assert.equal(result.pack.manifest.id, "mini");
    assert.equal(result.pack.units.length, 1);
    assert.equal(result.pack.traits.length, 1);
    assert.match(result.modHash, /^[0-9a-f]{64}$/);
  }
});
