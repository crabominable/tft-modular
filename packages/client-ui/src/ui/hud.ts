import type { MatchSnapshot, PluginBundle } from "../types.ts";
import { computeTraits } from "../traits.ts";

export function renderTopBar(
  el: HTMLElement,
  snap: MatchSnapshot,
  bundle: PluginBundle,
  humanGold: number,
  humanHp: number,
  humanLevel: number,
): void {
  const phaseLabel =
    snap.phase === "shop"
      ? "PLANEJAMENTO"
      : snap.phase === "combat"
        ? "COMBATE"
        : "FIM DE PARTIDA";

  el.innerHTML = `
    <div class="brand">${bundle.manifest.name.toUpperCase()}</div>
    <div class="stage">RODADA <em>${snap.round}</em> · ${phaseLabel}</div>
    <div class="eco-cluster" aria-label="Economia e vida">
      <div class="eco-stat gold" title="Ouro">
        <span class="lbl">Ouro</span>
        <span class="val">${humanGold}</span>
      </div>
      <div class="eco-stat hp" title="Vida">
        <span class="lbl">Vida</span>
        <span class="val">${humanHp}</span>
      </div>
      <div class="eco-stat lvl" title="Nível">
        <span class="lbl">Nível</span>
        <span class="val">${humanLevel}</span>
      </div>
    </div>
  `;
}

export function renderTraits(el: HTMLElement, snap: MatchSnapshot, bundle: PluginBundle): void {
  const human = snap.players[0];
  const traits = computeTraits(human, bundle);
  if (traits.length === 0) {
    el.innerHTML = `<h3>Sinergias</h3><p style="color:var(--muted);font-size:0.85rem;padding:0 0.35rem;font-weight:600">Coloque unidades no tabuleiro para ativar traits.</p>`;
    return;
  }
  el.innerHTML =
    `<h3>Sinergias</h3>` +
    traits
      .map((t) => {
        const on = t.activeTier.length > 0;
        const next = t.nextAt != null ? ` / ${t.nextAt}` : "";
        const initials = t.def.name
          .split(/\s+/)
          .map((w) => w[0])
          .join("")
          .slice(0, 2)
          .toUpperCase();
        return `
          <div class="trait-row ${on ? "active" : ""}">
            <div class="trait-icon">${initials}</div>
            <div class="trait-meta">
              <div class="trait-name">${t.def.name}</div>
              <div class="trait-count">${t.count}${next}${on ? " · ATIVO" : ""}</div>
            </div>
          </div>`;
      })
      .join("");
}

export function renderFoes(el: HTMLElement, snap: MatchSnapshot): void {
  const maxHp = 20;
  const cards = snap.players
    .map((p, i) => {
      const label = i === 0 ? "Você" : "Rival IA";
      const pct = Math.max(0, Math.min(100, (p.hp / maxHp) * 100));
      return `
        <div class="foe-card ${i === 0 ? "you" : ""}">
          <div class="name">${label}</div>
          <div class="hp-bar"><i style="width:${pct}%"></i></div>
          <div class="hp-num">${p.hp} HP · Nv ${p.level} · ${p.gold} ouro</div>
        </div>`;
    })
    .join("");
  el.innerHTML = `<h3>Lobby</h3>${cards}`;
}
