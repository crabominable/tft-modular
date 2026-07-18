import type { PlayerSnapshot } from "../types.ts";

export type BoardHandlers = {
  onSelectBench: (instanceId: number) => void;
  onSelectCell: (x: number, y: number) => void;
};

export function renderBoard(
  root: HTMLElement,
  human: PlayerSnapshot | undefined,
  unitNames: Map<string, string>,
  selectedInstanceId: number | null,
  handlers: BoardHandlers,
): void {
  const board = human?.board ?? [];
  const bench = human?.bench ?? [];

  const cells: string[] = [];
  for (let y = 0; y < 2; y++) {
    for (let x = 0; x < 4; x++) {
      const unit = board.find((b) => b.cell[0] === x && b.cell[1] === y);
      const label = unit
        ? `${unitNames.get(unit.def_id) ?? unit.def_id}<br/><code>#${unit.instance_id}</code>`
        : `<span class="meta">(${x},${y})</span>`;
      cells.push(
        `<div class="cell" data-x="${x}" data-y="${y}">${label}</div>`,
      );
    }
  }

  const benchHtml =
    bench.length === 0
      ? `<span class="meta">empty</span>`
      : bench
          .map((u) => {
            const sel = selectedInstanceId === u.instance_id ? " selected" : "";
            const name = unitNames.get(u.def_id) ?? u.def_id;
            return `<button type="button" class="bench-item${sel}" data-id="${u.instance_id}">${name} #${u.instance_id} (${u.cost}g)</button>`;
          })
          .join("");

  root.innerHTML = `
    <h2>Board (you)</h2>
    <p class="meta">Select a bench unit, then click a cell to place. Or use Auto-place.</p>
    <div class="board-grid">${cells.join("")}</div>
    <h2 style="margin-top:0.75rem">Bench</h2>
    <div class="bench-list">${benchHtml}</div>
  `;

  root.querySelectorAll<HTMLElement>(".cell").forEach((el) => {
    el.addEventListener("click", () => {
      handlers.onSelectCell(Number(el.dataset.x), Number(el.dataset.y));
    });
  });
  root.querySelectorAll<HTMLButtonElement>(".bench-item").forEach((btn) => {
    btn.addEventListener("click", () => {
      handlers.onSelectBench(Number(btn.dataset.id));
    });
  });
}
