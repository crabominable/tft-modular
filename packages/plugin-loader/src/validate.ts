import { createRequire } from "node:module";
import { readFileSync } from "node:fs";
import Ajv2020 from "ajv/dist/2020.js";
import addFormats from "ajv-formats";
import type { ErrorObject, ValidateFunction } from "ajv";

const require = createRequire(import.meta.url);

function loadSchema(name: string): object {
  const resolved = require.resolve(`@tft/plugin-schema/schemas/${name}`);
  return JSON.parse(readFileSync(resolved, "utf8")) as object;
}

const ajv = new Ajv2020({
  allErrors: true,
  strict: false,
  validateSchema: false,
});
addFormats(ajv);

const validateManifestFn = ajv.compile(loadSchema("manifest.schema.json"));
const validateUnitFn = ajv.compile(loadSchema("unit.schema.json"));
const validateTraitFn = ajv.compile(loadSchema("trait.schema.json"));
const validateAbilityFn = ajv.compile(loadSchema("ability.schema.json"));

export type ValidationResult =
  | { ok: true; value: unknown }
  | { ok: false; errors: string[] };

function formatErrors(errors: ErrorObject[] | null | undefined): string[] {
  if (!errors || errors.length === 0) return ["validation failed"];
  return errors.map((e) => {
    const path = e.instancePath || "/";
    return `${path}: ${e.message ?? "invalid"}`;
  });
}

function run(validator: ValidateFunction, data: unknown): ValidationResult {
  if (validator(data)) {
    return { ok: true, value: data };
  }
  return { ok: false, errors: formatErrors(validator.errors) };
}

export function validateManifestJson(data: unknown): ValidationResult {
  return run(validateManifestFn, data);
}

export function validateUnitJson(data: unknown): ValidationResult {
  return run(validateUnitFn, data);
}

export function validateTraitJson(data: unknown): ValidationResult {
  return run(validateTraitFn, data);
}

export function validateAbilityJson(data: unknown): ValidationResult {
  return run(validateAbilityFn, data);
}
