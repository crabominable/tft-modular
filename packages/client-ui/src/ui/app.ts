import type { WasmMatch } from "../bridge.ts";
import type { MatchSnapshot, PluginBundle } from "../types.ts";
import { renderBoard } from "./board.ts";
import { renderHud } from "./hud.ts";
import { renderShop } from "./shop.ts";

const HUMAN = 0;

export class App {
  private match: WasmMatch;
  private unitNames: Map<string, string>;
  private modHash: string | undefined;
  private engineVer: string;
  private lastEvents: unknown[] = [];
  private lastError: string | null = null;
  private selectedInstanceId: number | null = null;

  private hudEl: HTMLElement;
  private shopEl: HTMLElement;
  private boardEl: HTMLElement;
  private logEl: HTMLElement;
  private errEl: HTMLElement;

  constructor(
    root: HTMLElement,
    match: WasmMatch,
    bundle: PluginBundle,
    engineVer: string,
  ) {
    this.match = match;
    this.engineVer = engineVer;
    this.modHash = bundle.modHash;
    this.unitNames = new Map(bundle.units.map((u) => [u.id, u.name]));

    root.innerHTML = `
      <section id="hud"></section>
      <section id="shop"></section>
      <section id="board"></section>
      <section>
        <h2>Events</h2>
        <div id="err" class="error"></div>
        <pre id="log">[]</pre>
      </section>
    `;
    this.hudEl = root.querySelector("#hud")!;
    this.shopEl = root.querySelector("#shop")!;
    this.boardEl = root.querySelector("#board")!;
    this.logEl = root.querySelector("#log")!;
    this.errEl = root.querySelector("#err")!;
    this.refresh();
  }

  private snapshot(): MatchSnapshot {
    return JSON.parse(this.match.snapshot_json()) as MatchSnapshot;
  }

  private applyCmd(cmd: object): void {
    this.lastError = null;
    try {
      const raw = this.match.apply(HUMAN, JSON.stringify(cmd));
      this.lastEvents = JSON.parse(raw) as unknown[];
    } catch (err) {
      this.lastError = err instanceof Error ? err.message : String(err);
      this.lastEvents = [];
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
    // Place each bench unit onto the first empty cell until board/bench empty.
    // Re-read snapshot after each place (instance list shrinks).
    for (let guard = 0; guard < 16; guard++) {
      const snap = this.snapshot();
      if (snap.phase !== "shop") break;
      const human = snap.players[0];
      if (!human || human.bench.length === 0) break;
      const cell = this.firstEmptyCell(snap);
      if (!cell) break;
      const id = human.bench[0]!.instance_id;
      this.applyCmd({
        type: "place_unit",
        unit_instance_id: id,
        cell,
      });
      if (this.lastError) break;
    }
  }

  private refresh(): void {
    const snap = this.snapshot();
    const human = snap.players[0];
    const shopOpen = snap.phase === "shop" && !(human?.shop_ready ?? true);
    const hexHash = this.match.state_hash();

    renderHud(this.hudEl, snap, hexHash, this.modHash, this.engineVer);
    renderShop(this.shopEl, human, this.unitNames, shopOpen, {
      onBuy: (i) => this.applyCmd({ type: "buy_unit", shop_index: i }),
      onReroll: () => this.applyCmd({ type: "reroll" }),
      onBuyXp: () => this.applyCmd({ type: "buy_exp" }),
      onEndShop: () => this.applyCmd({ type: "end_shop_phase" }),
      onAutoPlace: () => this.autoPlace(),
    });
    renderBoard(this.boardEl, human, this.unitNames, this.selectedInstanceId, {
      onSelectBench: (id) => {
        this.selectedInstanceId =
          this.selectedInstanceId === id ? null : id;
        this.refresh();
      },
      onSelectCell: (x, y) => {
        if (this.selectedInstanceId == null) {
          this.lastError = "Select a bench unit first (or use Auto-place).";
          this.errEl.textContent = this.lastError;
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
    });

    this.errEl.textContent = this.lastError ?? "";
    this.logEl.textContent = JSON.stringify(this.lastEvents, null, 2);
  }
}
