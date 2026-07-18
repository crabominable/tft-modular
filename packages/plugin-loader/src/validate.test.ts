import test from "node:test";
import assert from "node:assert/strict";
import { validateUnitJson, validateAbilityJson } from "./validate.ts";
import { loadPackFromFiles } from "./load.ts";

test("valid unit passes", () => {
  const unit = {
    id: "ember_scout",
    name: "Ember Scout",
    cost: 1,
    traits: ["emberkin"],
    ability_id: "spark_cut",
    stats: { hp: 500, atk: 45, armor: 10, mr: 10, range: 1, attack_speed_milli: 700 },
  };
  assert.equal(validateUnitJson(unit).ok, true);
});

test("unknown ability effect type fails at pack link stage later; unit schema still ok without ability body", () => {
  const bad = {
    id: "x",
    name: "X",
    cost: 1,
    traits: [],
    stats: { hp: 1, atk: 1, range: 1, attack_speed_milli: 100 },
  };
  // missing nothing required — ok
  assert.equal(validateUnitJson(bad).ok, true);
});

test("unit cost out of range fails schema", () => {
  const unit = {
    id: "x",
    name: "X",
    cost: 9,
    traits: [],
    stats: { hp: 1, atk: 1, range: 1, attack_speed_milli: 100 },
  };
  assert.equal(validateUnitJson(unit).ok, false);
});

test("invalid ability effect type fails schema", () => {
  const ability = {
    id: "boom",
    name: "Boom",
    trigger: "ON_CAST",
    targeting: { type: "SELF" },
    effects: [{ type: "NUKE_EVERYTHING", amount: 1 }],
  };
  assert.equal(validateAbilityJson(ability).ok, false);
});

test("pack with bad cost is rejected at load", async () => {
  const enc = new TextEncoder();
  const files = new Map<string, Uint8Array>([
    [
      "manifest.json",
      enc.encode(
        JSON.stringify({
          id: "bad_pack",
          name: "Bad",
          version: "0.1.0",
          api_version: "1.0.0",
        }),
      ),
    ],
    [
      "units/x.json",
      enc.encode(
        JSON.stringify({
          id: "x",
          name: "X",
          cost: 9,
          traits: [],
          stats: { hp: 1, atk: 1, range: 1, attack_speed_milli: 100 },
        }),
      ),
    ],
  ]);
  const result = await loadPackFromFiles(files);
  assert.equal(result.ok, false);
});
