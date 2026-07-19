import type { PlayerSnapshot, PluginBundle } from "../types.ts";
import { COST_COLORS } from "../types.ts";

export type BoardHandlers = {
  onSelectBench: (instanceId: number) => void;
  onSelectCell: (x: number, y: number) => void;
};

function unitVisual(
  defId: string,
  cost: number,
  names: Map<string, string>,
): string {
  const c = COST_COLORS[cost] ?? COST_COLORS[1]!;
  const name = names.get(defId) ?? defId;
  const initial = name
    .split(/\s+/)
    .map((w) => w[0])
    .join("")
    .slice(0, 2)
    .toUpperCase();
  return `
    <div class="unit-chip">
      <div class="portrait" style="border-color:${c.border};box-shadow:0 0 14px ${c.glow};color:${c.label}">${initial}</div>
      <div class="name" style="color:${c.label}">${name}</div>
      <div class="cost">${cost} ouro</div>
    </div>`;
}

export function renderArena(
  root: HTMLElement,
  human: PlayerSnapshot | undefined,
  ai: PlayerSnapshot | undefined,
  bundle: PluginBundle,
  selectedInstanceId: number | null,
  phase: string,
  handlers: BoardHandlers,
): void {
  const names = new Map(bundle.units.map((u) => [u.id, u.name]));
  const board = human?.board ?? [];
  const enemyBoard = ai?.board ?? [];
  const bench = human?.bench ?? [];

  // Enemy: reverse y so front line sits toward mid-field
  const enemyCells: string[] = [];
  for (let y = 1; y >= 0; y--) {
    for (let x = 0; x < 4; x++) {
      const unit = enemyBoard.find((b) => b.cell[0] === x && b.cell[1] === y);
      if (unit) {
        enemyCells.push(
          `<div class="hex-cell enemy-filled">${unitVisual(unit.def_id, unit.cost, names)}</div>`,
        );
      } else {
        enemyCells.push(`<div class="hex-cell" aria-hidden="true"></div>`);
      }
    }
  }

  const playerCells: string[] = [];
  for (let y = 0; y < 2; y++) {
    for (let x = 0; x < 4; x++) {
      const unit = board.find((b) => b.cell[0] === x && b.cell[1] === y);
      const filled = unit ? " filled" : "";
      const inner = unit ? unitVisual(unit.def_id, unit.cost, names) : "";
      playerCells.push(
        `<div class="hex-cell${filled}" data-x="${x}" data-y="${y}" role="button" tabindex="0">${inner}</div>`,
      );
    }
  }

  const benchSlots: string[] = [];
  for (let i = 0; i < 9; i++) {
    const u = bench[i];
    if (!u) {
      benchSlots.push(`<div class="bench-slot empty">—</div>`);
      continue;
    }
    const c = COST_COLORS[u.cost] ?? COST_COLORS[1]!;
    const name = names.get(u.def_id) ?? u.def_id;
    const sel = selectedInstanceId === u.instance_id ? " selected" : "";
    benchSlots.push(
      `<button type="button" class="bench-slot${sel}" data-id="${u.instance_id}" style="border-color:${c.border}">${name}</button>`,
    );
  }

  const bannerClass = phase === "combat" ? "combat" : "";
  const bannerText =
    phase === "combat" ? "COMBATE" : phase === "match_end" ? "FIM" : "PLANEJE";

  const over =
    phase === "match_end" ? `<div class="match-over">FIM DE PARTIDA</div>` : "";

  // Flex stack: enemy top, player bottom, bench under — no absolute overlap
  root.innerHTML = `
    <div class="hex-field">
      <div class="phase-banner ${bannerClass}">${bannerText}</div>
      <div class="board-stack">
        <div class="board-label enemy">Rival</div>
        <div class="board-side enemy">${enemyCells.join("")}</div>
        <div class="board-label player">Você</div>
        <div class="board-side player">${playerCells.join("")}</div>
      </div>
      <div class="bench-row">${benchSlots.join("")}</div>
      ${over}
    </div>
  `;

  root.querySelectorAll<HTMLElement>(".board-side.player .hex-cell").forEach((el) => {
    el.addEventListener("click", () => {
      if (el.dataset.x == null) return;
      handlers.onSelectCell(Number(el.dataset.x), Number(el.dataset.y));
    });
  });
  root.querySelectorAll<HTMLButtonElement>(".bench-slot[data-id]").forEach((btn) => {
    btn.addEventListener("click", () => {
      handlers.onSelectBench(Number(btn.dataset.id));
    });
  });
}
