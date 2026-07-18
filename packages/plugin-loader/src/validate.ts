import Ajv2020 from "ajv/dist/2020.js";
import addFormats from "ajv-formats";
import type { ErrorObject, ValidateFunction } from "ajv";

import abilitySchema from "@tft/plugin-schema/schemas/ability.schema.json" with { type: "json" };
import manifestSchema from "@tft/plugin-schema/schemas/manifest.schema.json" with { type: "json" };
import traitSchema from "@tft/plugin-schema/schemas/trait.schema.json" with { type: "json" };
import unitSchema from "@tft/plugin-schema/schemas/unit.schema.json" with { type: "json" };

const ajv = new Ajv2020({
  allErrors: true,
  strict: false,
  validateSchema: false,
});
addFormats(ajv);

const validateManifestFn = ajv.compile(manifestSchema as object);
const validateUnitFn = ajv.compile(unitSchema as object);
const validateTraitFn = ajv.compile(traitSchema as object);
const validateAbilityFn = ajv.compile(abilitySchema as object);

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
