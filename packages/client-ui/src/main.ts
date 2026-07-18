import { createMatch, engineVersion, QA_SEED } from "./bridge.ts";
import {
  loadBundleFromCache,
  loadBundleFromFile,
  loadBundleFromUrl,
  loadDefaultReference,
} from "./modLoad.ts";
import type { PluginBundle } from "./types.ts";
import { App } from "./ui/app.ts";

const appRoot = document.getElementById("app");
if (!appRoot) {
  throw new Error("#app missing");
}

async function startMatch(bundle: PluginBundle, sourceLabel: string): Promise<void> {
  appRoot!.textContent = `Starting match (${sourceLabel})…`;

  const { modHash: _modHash, ...plugin } = bundle;

  const [match, ver] = await Promise.all([
    createMatch(plugin, QA_SEED),
    engineVersion(),
  ]);

  appRoot!.replaceChildren();
  new App(appRoot!, match, bundle, ver);
}

function showLoader(): void {
  appRoot!.innerHTML = `
    <section id="loader">
      <h2>Load mod pack</h2>
      <p class="meta" style="margin:0 0 0.75rem">
        Zip upload or URL · validated client-side · cached locally by content hash (no rehost)
      </p>
      <div class="row" style="margin-bottom:0.75rem">
        <button type="button" id="btn-default">Load reference-mod</button>
      </div>
      <div class="row" style="margin-bottom:0.75rem">
        <label>
          Zip file
          <input type="file" id="file-zip" accept=".zip,application/zip" />
        </label>
      </div>
      <div class="row" style="margin-bottom:0.5rem">
        <label style="flex:1; min-width:12rem">
          Pack URL
          <input type="url" id="url-input" placeholder="https://…/pack.zip" style="width:100%; box-sizing:border-box" />
        </label>
      </div>
      <div class="row" style="margin-bottom:0.75rem">
        <label style="flex:1; min-width:12rem">
          Expected hash (optional — cache hit skips network)
          <input type="text" id="hash-input" placeholder="64 hex chars" spellcheck="false" style="width:100%; box-sizing:border-box; font-family:ui-monospace,monospace" />
        </label>
      </div>
      <div class="row">
        <button type="button" id="btn-url">Load from URL</button>
        <button type="button" class="secondary" id="btn-cache">Load from local cache</button>
      </div>
      <p id="load-status" class="meta" style="margin:0.75rem 0 0"></p>
      <p id="load-error" class="error" style="margin:0.5rem 0 0"></p>
    </section>
  `;

  const statusEl = appRoot!.querySelector("#load-status") as HTMLElement;
  const errEl = appRoot!.querySelector("#load-error") as HTMLElement;
  const fileInput = appRoot!.querySelector("#file-zip") as HTMLInputElement;
  const urlInput = appRoot!.querySelector("#url-input") as HTMLInputElement;
  const hashInput = appRoot!.querySelector("#hash-input") as HTMLInputElement;

  const setBusy = (msg: string) => {
    errEl.textContent = "";
    statusEl.textContent = msg;
  };
  const setErr = (err: unknown) => {
    statusEl.textContent = "";
    errEl.textContent = err instanceof Error ? err.message : String(err);
  };

  appRoot!.querySelector("#btn-default")!.addEventListener("click", () => {
    setBusy("Loading reference-mod…");
    loadDefaultReference()
      .then((b) => startMatch(b, "reference-mod"))
      .catch(setErr);
  });

  fileInput.addEventListener("change", () => {
    const file = fileInput.files?.[0];
    if (!file) return;
    setBusy(`Reading ${file.name}…`);
    loadBundleFromFile(file)
      .then((b) => startMatch(b, `zip:${file.name}`))
      .catch(setErr);
  });

  appRoot!.querySelector("#btn-url")!.addEventListener("click", () => {
    const url = urlInput.value.trim();
    if (!url) {
      setErr(new Error("Enter a pack URL"));
      return;
    }
    const hash = hashInput.value.trim() || undefined;
    setBusy(hash ? "Checking cache / fetching URL…" : "Fetching URL…");
    loadBundleFromUrl(url, hash)
      .then(({ bundle, fromCache }) =>
        startMatch(bundle, fromCache ? `cache:${bundle.modHash?.slice(0, 12)}` : `url`),
      )
      .catch(setErr);
  });

  appRoot!.querySelector("#btn-cache")!.addEventListener("click", () => {
    const hash = hashInput.value.trim();
    if (!hash) {
      setErr(new Error("Enter a content hash to load from local cache"));
      return;
    }
    setBusy("Reading local cache…");
    loadBundleFromCache(hash)
      .then((b) => startMatch(b, `cache:${hash.slice(0, 12)}`))
      .catch(setErr);
  });
}

async function boot(): Promise<void> {
  // Prefer interactive loader; auto-start reference-mod when ?autoload=1
  const params = new URLSearchParams(location.search);
  if (params.get("autoload") === "1" || params.get("autoload") === "true") {
    appRoot!.textContent = "Loading reference-mod + wasm…";
    const bundle = await loadDefaultReference();
    await startMatch(bundle, "reference-mod");
    return;
  }
  showLoader();
}

boot().catch((err) => {
  const msg = err instanceof Error ? err.message : String(err);
  appRoot!.innerHTML = `<p class="error">Boot failed: ${msg}</p>`;
  console.error(err);
});
