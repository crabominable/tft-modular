import type { PlayerSnapshot, PluginBundle } from "../types.ts";
import { COST_COLORS } from "../types.ts";

export type ShopHandlers = {
  onBuy: (shopIndex: number) => void;
  onReroll: () => void;
  onBuyXp: () => void;
  onEndShop: () => void;
  onAutoPlace: () => void;
};

function xpProgress(level: number, xp: number): number {
  const gaps: Record<number, number> = {
    1: 2,
    2: 4,
    3: 4,
    4: 10,
    5: 16,
    6: 20,
    7: 24,
    8: 20,
  };
  const gap = gaps[level] ?? 20;
  return Math.min(100, Math.round(((xp % Math.max(gap, 1)) / gap) * 100));
}

export function renderShopDock(
  root: HTMLElement,
  human: PlayerSnapshot | undefined,
  bundle: PluginBundle,
  shopOpen: boolean,
  handlers: ShopHandlers,
): void {
  const names = new Map(bundle.units.map((u) => [u.id, u.name]));
  const traitNames = new Map(bundle.traits.map((t) => [t.id, t.name]));
  const slots = human?.shop ?? [];
  const level = human?.level ?? 1;
  const xp = human?.xp ?? 0;
  const gold = human?.gold ?? 0;

  const cards = [0, 1, 2, 3, 4]
    .map((i) => {
      const offer = slots[i] ?? null;
      if (!offer) {
        return `<div class="shop-card empty"><div class="art">?</div><div class="title">Vazio</div></div>`;
      }
      const unit = bundle.units.find((u) => u.id === offer.def_id);
      const c = COST_COLORS[offer.cost] ?? COST_COLORS[1]!;
      const name = names.get(offer.def_id) ?? offer.def_id;
      const traits = (unit?.traits ?? [])
        .map((t) => traitNames.get(t) ?? t)
        .join(" · ");
      const initial = name
        .split(/\s+/)
        .map((w) => w[0])
        .join("")
        .slice(0, 2)
        .toUpperCase();
      const canBuy = shopOpen && gold >= offer.cost;
      return `
        <div class="shop-card" style="border-color:${c.border};box-shadow:0 0 18px ${c.glow}">
          <span class="cost-tag" style="color:${c.label};border-color:${c.border}">${offer.cost}</span>
          <div class="art" style="color:${c.label};border-color:${c.border};box-shadow:inset 0 0 24px ${c.glow}">${initial}</div>
          <div class="title" style="color:${c.label}">${name}</div>
          <div class="traits-line">${traits || "—"}</div>
          <button type="button" class="buy" data-buy="${i}" ${canBuy ? "" : "disabled"}>
            Comprar · ${offer.cost} ouro
          </button>
        </div>`;
    })
    .join("");

  const pct = xpProgress(level, xp);

  root.innerHTML = `
    <div class="shop-controls">
      <div class="shop-gold-block" title="Seu ouro">
        <div class="coin" aria-hidden="true"></div>
        <div class="g-meta">
          <span class="g-lbl">Ouro</span>
          <span class="g-val">${gold}</span>
        </div>
      </div>
      <div class="lvl-block">Nível <strong>${level}</strong> · XP ${xp}</div>
      <div class="xp-track"><i style="width:${pct}%"></i></div>
      <button type="button" class="btn-tft" data-act="buyxp" ${shopOpen && gold >= 4 ? "" : "disabled"}>XP · 4 ouro</button>
      <button type="button" class="btn-tft gold-btn" data-act="reroll" ${shopOpen && gold >= 2 ? "" : "disabled"}>Reroll · 2 ouro</button>
    </div>
    <div class="shop-row">${cards}</div>
    <div class="shop-end">
      <button type="button" class="btn-tft" data-act="autoplace" ${shopOpen ? "" : "disabled"}>Auto-campo</button>
      <button type="button" class="btn-tft ready" data-act="endshop" ${shopOpen ? "" : "disabled"}>Pronto</button>
    </div>
  `;

  root.querySelectorAll<HTMLButtonElement>("[data-buy]").forEach((btn) => {
    btn.addEventListener("click", () => handlers.onBuy(Number(btn.dataset.buy)));
  });
  root.querySelector<HTMLButtonElement>('[data-act="reroll"]')?.addEventListener("click", handlers.onReroll);
  root.querySelector<HTMLButtonElement>('[data-act="buyxp"]')?.addEventListener("click", handlers.onBuyXp);
  root.querySelector<HTMLButtonElement>('[data-act="endshop"]')?.addEventListener("click", handlers.onEndShop);
  root.querySelector<HTMLButtonElement>('[data-act="autoplace"]')?.addEventListener("click", handlers.onAutoPlace);
}
