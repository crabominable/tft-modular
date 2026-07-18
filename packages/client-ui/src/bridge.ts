import init, { WasmMatch, wasm_engine_version } from "./wasm-pkg/engine_wasm.js";

let initPromise: Promise<void> | null = null;

async function ensureInit(): Promise<void> {
  if (!initPromise) {
    initPromise = init().then(() => undefined);
  }
  await initPromise;
}

/** Fixed seed for QA / deterministic manual testing. */
export const QA_SEED = 42n;

export async function createMatch(
  pluginBundle: unknown,
  seed: bigint | number = QA_SEED,
): Promise<WasmMatch> {
  await ensureInit();
  const seedBig = typeof seed === "bigint" ? seed : BigInt(seed);
  return new WasmMatch(JSON.stringify(pluginBundle), seedBig);
}

export async function engineVersion(): Promise<string> {
  await ensureInit();
  return wasm_engine_version();
}

export type { WasmMatch };
