/**
 * Bundle a data mod into packages/client-ui/public/reference-mod.json
 *
 * Usage (from repo root):
 *   node --experimental-strip-types tools/bundle-mod.mjs
 *   node --experimental-strip-types tools/bundle-mod.mjs shinobi-leaf
 */
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { loadPackFromDirectory } from "../packages/plugin-loader/src/index.ts";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(__dirname, "..");
const modName = process.argv[2] || "shinobi-leaf";
const modDir = path.join(root, "mods", modName);
const outPath = path.join(
  root,
  "packages",
  "client-ui",
  "public",
  "reference-mod.json",
);

if (!fs.existsSync(modDir)) {
  console.error(`Mod not found: ${modDir}`);
  process.exit(1);
}

const result = await loadPackFromDirectory(modDir);
if (!result.ok) {
  console.error("bundle-mod failed:", result.error);
  process.exit(1);
}

const { pack, modHash } = result;
const payload = {
  ...pack,
  modHash,
};

fs.mkdirSync(path.dirname(outPath), { recursive: true });
fs.writeFileSync(outPath, JSON.stringify(payload, null, 2) + "\n", "utf8");

console.log(`Wrote ${path.relative(root, outPath)} from mods/${modName}`);
console.log(`modHash=${modHash}`);
console.log(
  `units=${pack.units.length} traits=${pack.traits.length} abilities=${pack.abilities.length}`,
);
