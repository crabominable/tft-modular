import test from "node:test";
import assert from "node:assert/strict";
import { isSafePackPath } from "./zip.ts";
import { assertWithinLimits } from "./limits.ts";
import { MAX_PACK_BYTES, MAX_UNITS } from "@tft/plugin-schema";

test("rejects parent traversal", () => {
  assert.equal(isSafePackPath("../secret.json"), false);
  assert.equal(isSafePackPath("units/../../x"), false);
});

test("accepts normal relative paths", () => {
  assert.equal(isSafePackPath("units/ember_scout.json"), true);
});

test("rejects absolute, drive, backslash, empty segments", () => {
  assert.equal(isSafePackPath("/etc/passwd"), false);
  assert.equal(isSafePackPath("C:\\windows\\x"), false);
  assert.equal(isSafePackPath("units//double.json"), false);
  assert.equal(isSafePackPath(""), false);
  assert.equal(isSafePackPath("units/./x.json"), false);
});

test("assertWithinLimits rejects oversize pack and counts", () => {
  assert.throws(
    () =>
      assertWithinLimits(
        { units: 1, traits: 1, abilities: 1 },
        MAX_PACK_BYTES + 1,
      ),
    /MAX_PACK_BYTES|PackTooLarge|bytes/i,
  );
  assert.throws(
    () =>
      assertWithinLimits(
        { units: MAX_UNITS + 1, traits: 0, abilities: 0 },
        100,
      ),
    /MAX_UNITS|units/i,
  );
});
