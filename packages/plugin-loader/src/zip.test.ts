import test from "node:test";
import assert from "node:assert/strict";
import { zipSync } from "fflate";
import { isSafePackPath, unzipPackBytes } from "./zip.ts";
import { loadPackFromZip } from "./load.ts";

test("rejects parent traversal in zip paths", () => {
  assert.equal(isSafePackPath("../x"), false);
});

test("unzipPackBytes rejects unsafe paths", () => {
  const zipped = zipSync({
    "../secret.json": new TextEncoder().encode("{}"),
  });
  const result = unzipPackBytes(zipped);
  assert.equal(result.ok, false);
});

test("loadPackFromZip rejects pack with unsafe path", async () => {
  const enc = new TextEncoder();
  const zipped = zipSync({
    "manifest.json": enc.encode(
      JSON.stringify({
        id: "z",
        name: "Z",
        version: "0.1.0",
        api_version: "1.0.0",
      }),
    ),
    "../x": enc.encode("{}"),
  });
  const result = await loadPackFromZip(zipped);
  assert.equal(result.ok, false);
});

test("loadPackFromZip accepts minimal valid pack", async () => {
  const enc = new TextEncoder();
  const zipped = zipSync({
    "manifest.json": enc.encode(
      JSON.stringify({
        id: "mini",
        name: "Mini",
        version: "0.1.0",
        api_version: "1.0.0",
      }),
    ),
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
  });
  const result = await loadPackFromZip(zipped);
  assert.equal(result.ok, true);
  if (result.ok) {
    assert.equal(result.pack.manifest.id, "mini");
    assert.match(result.modHash, /^[0-9a-f]{64}$/);
  }
});
