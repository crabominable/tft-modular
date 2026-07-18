import type { MatchSnapshot, PlayerSnapshot } from "../types.ts";

export function renderHud(
  root: HTMLElement,
  snap: MatchSnapshot,
  hexHash: string,
  modHash: string | undefined,
  engineVer: string,
): void {
  const human = snap.players[0] as PlayerSnapshot | undefined;
  const ai = snap.players[1] as PlayerSnapshot | undefined;

  root.innerHTML = `
    <h2>Match</h2>
    <div class="row">
      <span>Phase: <strong>${snap.phase}</strong></span>
      <span>Round: <strong>${snap.round}</strong></span>
      <span>Winner: <strong>${snap.winner_player ?? "—"}</strong></span>
    </div>
    <div class="row" style="margin-top:0.4rem">
      <span>You HP <strong>${human?.hp ?? "—"}</strong></span>
      <span>Gold <strong>${human?.gold ?? "—"}</strong></span>
      <span>Lvl <strong>${human?.level ?? "—"}</strong></span>
      <span>XP <strong>${human?.xp ?? "—"}</strong></span>
      <span>AI HP <strong>${ai?.hp ?? "—"}</strong></span>
    </div>
    <div class="row" style="margin-top:0.4rem; font-size:0.85rem; color:#9aa7b5">
      <span>state_hash <code>${hexHash}</code></span>
      <span>engine <code>${engineVer}</code></span>
      ${modHash ? `<span>modHash <code>${modHash.slice(0, 16)}…</code></span>` : ""}
    </div>
  `;
}
