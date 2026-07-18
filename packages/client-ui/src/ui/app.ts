import type { WasmMatch } from "../bridge.ts";
import type { MatchSnapshot, PluginBundle } from "../types.ts";
import { renderArena } from "./board.ts";
import { renderFoes, renderTopBar, renderTraits } from "./hud.ts";
import { renderShopDock } from "./shop.ts";

const HUMAN = 0;

export class App {
  private match: WasmMatch;
  private bundle: PluginBundle;
  private lastError: string | null = null;
  private selectedInstanceId: number | null = null;

  private topEl: HTMLElement;
  private traitsEl: HTMLElement;
  private arenaEl: HTMLElement;
  private foesEl: HTMLElement;
  private shopEl: HTMLElement;
  private toastEl: HTMLElement | null = null;

  constructor(root: HTMLElement, match: WasmMatch, bundle: PluginBundle) {
    this.match = match;
    this.bundle = bundle;

    root.hidden = false;
    root.innerHTML = `
      <div class="game-shell">
        <header class="top-bar" id="top"></header>
        <aside class="traits-panel" id="traits"></aside>
        <main class="arena" id="arena"></main>
        <aside class="foes-panel" id="foes"></aside>
        <footer class="shop-dock" id="shop"></footer>
      </div>
      <details class="advanced">
        <summary>Mod avançado (zip / URL)</summary>
        <div class="panel">
          <p style="margin:0 0 0.4rem;color:var(--muted)">O jogo já carrega o pack oficial. Use só se quiser trocar o mod.</p>
          <label>Zip<input type="file" id="adv-file" accept=".zip,application/zip" /></label>
          <label>URL<input type="url" id="adv-url" placeholder="https://…/pack.zip" /></label>
          <button type="button" class="btn-tft" id="adv-reload" style="width:100%;margin-top:0.35rem">Recarregar reference-mod</button>
          <p id="adv-err" class="error" style="color:var(--danger);margin:0.4rem 0 0"></p>
        </div>
      </details>
    `;

    this.topEl = root.querySelector("#top")!;
    this.traitsEl = root.querySelector("#traits")!;
    this.arenaEl = root.querySelector("#arena")!;
    this.foesEl = root.querySelector("#foes")!;
    this.shopEl = root.querySelector("#shop")!;

    this.wireAdvanced(root);
    this.refresh();
  }

  private wireAdvanced(root: HTMLElement): void {
    const err = root.querySelector("#adv-err") as HTMLElement;
    root.querySelector("#adv-reload")?.addEventListener("click", () => {
      location.href = "/?autoload=1";
    });
    root.querySelector("#adv-file")?.addEventListener("change", async (ev) => {
      const input = ev.target as HTMLInputElement;
      const file = input.files?.[0];
      if (!file) return;
      try {
        const { loadBundleFromFile } = await import("../modLoad.ts");
        const { createMatch, QA_SEED } = await import("../bridge.ts");
        const bundle = await loadBundleFromFile(file);
        const { modHash: _, ...plugin } = bundle;
        const match = await createMatch(plugin, QA_SEED);
        this.match = match;
        this.bundle = bundle;
        this.selectedInstanceId = null;
        this.lastError = null;
        this.refresh();
      } catch (e) {
        err.textContent = e instanceof Error ? e.message : String(e);
      }
    });
  }

  private snapshot(): MatchSnapshot {
    return JSON.parse(this.match.snapshot_json()) as MatchSnapshot;
  }

  private toast(msg: string): void {
    this.toastEl?.remove();
    const el = document.createElement("div");
    el.className = "toast";
    el.textContent = msg;
    document.body.appendChild(el);
    this.toastEl = el;
    window.setTimeout(() => el.remove(), 2800);
  }

  private applyCmd(cmd: object): void {
    this.lastError = null;
    try {
      this.match.apply(HUMAN, JSON.stringify(cmd));
    } catch (err) {
      this.lastError = err instanceof Error ? err.message : String(err);
      this.toast(this.lastError);
    }
    this.refresh();
  }

  private firstEmptyCell(snap: MatchSnapshot): [number, number] | null {
    const board = snap.players[0]?.board ?? [];
    for (let y = 0; y < 2; y++) {
      for (let x = 0; x < 4; x++) {
        if (!board.some((b) => b.cell[0] === x && b.cell[1] === y)) {
          return [x, y];
        }
      }
    }
    return null;
  }

  private autoPlace(): void {
    for (let guard = 0; guard < 16; guard++) {
      const snap = this.snapshot();
      if (snap.phase !== "shop") break;
      const human = snap.players[0];
      if (!human || human.bench.length === 0) break;
      const cell = this.firstEmptyCell(snap);
      if (!cell) break;
      this.applyCmd({
        type: "place_unit",
        unit_instance_id: human.bench[0]!.instance_id,
        cell,
      });
      if (this.lastError) break;
    }
  }

  private refresh(): void {
    const snap = this.snapshot();
    const human = snap.players[0];
    const ai = snap.players[1];
    const shopOpen = snap.phase === "shop" && !(human?.shop_ready ?? true);

    renderTopBar(
      this.topEl,
      snap,
      this.bundle,
      human?.gold ?? 0,
      human?.hp ?? 0,
      human?.level ?? 1,
    );
    renderTraits(this.traitsEl, snap, this.bundle);
    renderFoes(this.foesEl, snap);
    renderArena(
      this.arenaEl,
      human,
      ai,
      this.bundle,
      this.selectedInstanceId,
      snap.phase,
      {
        onSelectBench: (id) => {
          this.selectedInstanceId =
            this.selectedInstanceId === id ? null : id;
          this.refresh();
        },
        onSelectCell: (x, y) => {
          if (this.selectedInstanceId == null) {
            this.toast("Selecione um shinobi no banco, depois o hexágono.");
            return;
          }
          const id = this.selectedInstanceId;
          this.selectedInstanceId = null;
          this.applyCmd({
            type: "place_unit",
            unit_instance_id: id,
            cell: [x, y],
          });
        },
      },
    );

    // Match over label
    if (snap.phase === "match_end") {
      const banner = this.arenaEl.querySelector(".match-over");
      if (banner) {
        const win =
          snap.winner_player === 0
            ? "VITÓRIA"
            : snap.winner_player === 1
              ? "DERROTA"
              : "EMPATE";
        banner.textContent = win;
      }
    }

    renderShopDock(this.shopEl, human, this.bundle, shopOpen, {
      onBuy: (i) => this.applyCmd({ type: "buy_unit", shop_index: i }),
      onReroll: () => this.applyCmd({ type: "reroll" }),
      onBuyXp: () => this.applyCmd({ type: "buy_exp" }),
      onEndShop: () => this.applyCmd({ type: "end_shop_phase" }),
      onAutoPlace: () => this.autoPlace(),
    });
  }
}
