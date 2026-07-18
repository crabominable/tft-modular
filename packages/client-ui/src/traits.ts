import type { PlayerSnapshot, PluginBundle, TraitDef } from "./types.ts";

export type ActiveTrait = {
  def: TraitDef;
  count: number;
  activeTier: number[];
  nextAt: number | null;
};

/** Unique trait ids on board (bench does not count in classic TFT; we use board only). */
export function computeTraits(
  human: PlayerSnapshot | undefined,
  bundle: PluginBundle,
): ActiveTrait[] {
  const board = human?.board ?? [];
  const counts = new Map<string, number>();
  const seenUnits = new Set<string>();

  for (const bu of board) {
    // One stack per distinct unit def (like unique champions), not per copy
    if (seenUnits.has(bu.def_id)) continue;
    seenUnits.add(bu.def_id);
    const unit = bundle.units.find((u) => u.id === bu.def_id);
    if (!unit) continue;
    for (const t of unit.traits) {
      counts.set(t, (counts.get(t) ?? 0) + 1);
    }
  }

  const out: ActiveTrait[] = [];
  for (const def of bundle.traits) {
    const count = counts.get(def.id) ?? 0;
    if (count === 0) continue;
    const thresholds = def.breakpoints.map((b) => b.min_units).sort((a, b) => a - b);
    const activeTier = thresholds.filter((t) => count >= t);
    const nextAt = thresholds.find((t) => t > count) ?? null;
    out.push({ def, count, activeTier, nextAt });
  }

  out.sort((a, b) => {
    const aOn = a.activeTier.length > 0 ? 1 : 0;
    const bOn = b.activeTier.length > 0 ? 1 : 0;
    if (aOn !== bOn) return bOn - aOn;
    return b.count - a.count;
  });
  return out;
}
