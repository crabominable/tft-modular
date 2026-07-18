import { createMatch, engineVersion, QA_SEED } from "./bridge.ts";
import type { PluginBundle } from "./types.ts";
import { App } from "./ui/app.ts";

const appRoot = document.getElementById("app");
if (!appRoot) {
  throw new Error("#app missing");
}

async function boot(): Promise<void> {
  appRoot!.textContent = "Loading reference-mod + wasm…";

  const res = await fetch("/reference-mod.json");
  if (!res.ok) {
    throw new Error(
      `Failed to fetch /reference-mod.json (${res.status}). Run: node --experimental-strip-types tools/bundle-mod.mjs`,
    );
  }
  const bundle = (await res.json()) as PluginBundle;

  // Strip host-only field before wasm (serde is tolerant of unknown fields,
  // but keep the plugin shape clean).
  const { modHash: _modHash, ...plugin } = bundle;

  const [match, ver] = await Promise.all([
    createMatch(plugin, QA_SEED),
    engineVersion(),
  ]);

  appRoot!.replaceChildren();
  new App(appRoot!, match, bundle, ver);
}

boot().catch((err) => {
  const msg = err instanceof Error ? err.message : String(err);
  appRoot!.innerHTML = `<p class="error">Boot failed: ${msg}</p>`;
  console.error(err);
});
