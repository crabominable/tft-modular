import { createMatch, QA_SEED } from "./bridge.ts";
import { loadDefaultReference } from "./modLoad.ts";
import type { PluginBundle } from "./types.ts";
import { App } from "./ui/app.ts";

const bootEl = document.getElementById("boot");
const appRoot = document.getElementById("app");
if (!appRoot) throw new Error("#app missing");

function setBoot(msg: string): void {
  const sub = bootEl?.querySelector(".boot-sub");
  if (sub) sub.textContent = msg;
}

async function startMatch(bundle: PluginBundle): Promise<void> {
  setBoot(`Iniciando ${bundle.manifest.name}…`);
  const { modHash: _m, ...plugin } = bundle;
  const match = await createMatch(plugin, QA_SEED);
  bootEl?.remove();
  appRoot!.hidden = false;
  new App(appRoot!, match, bundle);
}

async function boot(): Promise<void> {
  // Always auto-load the bundled pack — no form hell.
  // Advanced zip/URL lives in the in-game "Mod avançado" panel.
  setBoot("Baixando pack Shinobi Leaf…");
  const bundle = await loadDefaultReference();
  setBoot("Compilando partida (wasm)…");
  await startMatch(bundle);
}

boot().catch((err) => {
  const msg = err instanceof Error ? err.message : String(err);
  if (bootEl) {
    bootEl.innerHTML = `
      <div class="boot-card">
        <div class="boot-logo">ERRO</div>
        <div class="boot-sub" style="color:#ff8e8e;max-width:28rem">${msg}</div>
        <p style="color:var(--muted);font-size:0.85rem">Recarregue a página. Se persistir, o deploy pode estar sem reference-mod.json.</p>
      </div>`;
  }
  console.error(err);
});
