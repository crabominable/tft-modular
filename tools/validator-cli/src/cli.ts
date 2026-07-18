import path from "node:path";
import { loadPackFromDirectory } from "@tft/plugin-loader";

const args = process.argv.slice(2).filter((a) => a !== "--");
const dir = args[0];
if (!dir) {
  console.error("usage: validator-cli <mod-dir>");
  process.exit(2);
}

const result = await loadPackFromDirectory(path.resolve(process.cwd(), dir));
if (!result.ok) {
  console.error(result.error);
  process.exit(1);
}
console.log(`OK ${result.pack.manifest.id} hash=${result.modHash}`);
