import type { PlayerSnapshot } from "../types.ts";

export type ShopHandlers = {
  onBuy: (shopIndex: number) => void;
  onReroll: () => void;
  onBuyXp: () => void;
  onEndShop: () => void;
  onAutoPlace: () => void;
};

export function renderShop(
  root: HTMLElement,
  human: PlayerSnapshot | undefined,
  unitNames: Map<string, string>,
  shopOpen: boolean,
  handlers: ShopHandlers,
): void {
  const slots = human?.shop ?? [];
  const cards = [0, 1, 2, 3, 4]
    .map((i) => {
      const offer = slots[i] ?? null;
      if (!offer) {
        return `<div class="shop-card empty">Slot ${i}<br/>empty</div>`;
      }
      const name = unitNames.get(offer.def_id) ?? offer.def_id;
      return `
        <div class="shop-card">
          <div><strong>${name}</strong></div>
          <div class="meta">${offer.def_id} · ${offer.cost}g</div>
          <button data-buy="${i}" ${shopOpen ? "" : "disabled"}>Buy ${i}</button>
        </div>`;
    })
    .join("");

  root.innerHTML = `
    <h2>Shop</h2>
    <div class="row">${cards}</div>
    <div class="row" style="margin-top:0.6rem">
      <button class="secondary" data-act="reroll" ${shopOpen ? "" : "disabled"}>Reroll (2g)</button>
      <button class="secondary" data-act="buyxp" ${shopOpen ? "" : "disabled"}>Buy XP (4g)</button>
      <button class="secondary" data-act="autoplace" ${shopOpen ? "" : "disabled"}>Auto-place bench</button>
      <button class="danger" data-act="endshop" ${shopOpen ? "" : "disabled"}>End Shop</button>
    </div>
  `;

  root.querySelectorAll<HTMLButtonElement>("[data-buy]").forEach((btn) => {
    btn.addEventListener("click", () => {
      handlers.onBuy(Number(btn.dataset.buy));
    });
  });
  root.querySelector<HTMLButtonElement>('[data-act="reroll"]')?.addEventListener("click", handlers.onReroll);
  root.querySelector<HTMLButtonElement>('[data-act="buyxp"]')?.addEventListener("click", handlers.onBuyXp);
  root.querySelector<HTMLButtonElement>('[data-act="endshop"]')?.addEventListener("click", handlers.onEndShop);
  root.querySelector<HTMLButtonElement>('[data-act="autoplace"]')?.addEventListener("click", handlers.onAutoPlace);
}
