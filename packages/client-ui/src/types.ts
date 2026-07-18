/** Mirror of engine_core MatchSnapshot JSON. */

export type Phase = "shop" | "combat" | "match_end";

export type ShopOfferSnap = {
  def_id: string;
  cost: number;
};

export type OwnedUnitSnap = {
  instance_id: number;
  def_id: string;
  cost: number;
};

export type BoardUnit = {
  instance_id: number;
  def_id: string;
  cost: number;
  cell: [number, number];
};

export type PlayerSnapshot = {
  player_id: number;
  hp: number;
  gold: number;
  xp: number;
  level: number;
  shop_ready: boolean;
  shop: Array<ShopOfferSnap | null>;
  bench: OwnedUnitSnap[];
  board: BoardUnit[];
};

export type MatchSnapshot = {
  phase: Phase;
  round: number;
  players: PlayerSnapshot[];
  state_hash: number;
  winner_player: number | null;
};

export type TraitBreakpoint = {
  min_units: number;
  modifiers: Array<{ stat: string; amount: number }>;
};

export type TraitDef = {
  id: string;
  name: string;
  breakpoints: TraitBreakpoint[];
};

export type UnitDef = {
  id: string;
  name: string;
  cost: number;
  traits: string[];
  ability_id?: string | null;
  stats: Record<string, number>;
};

export type PluginBundle = {
  manifest: {
    id: string;
    name: string;
    version: string;
    api_version: string;
    description?: string;
  };
  units: UnitDef[];
  traits: TraitDef[];
  abilities: unknown[];
  modHash?: string;
};

export const COST_COLORS: Record<number, { border: string; glow: string; label: string }> = {
  1: { border: "#9aa4b2", glow: "rgba(154,164,178,0.35)", label: "#cfd6e0" },
  2: { border: "#3ecf6e", glow: "rgba(62,207,110,0.4)", label: "#8dffb0" },
  3: { border: "#4b9fff", glow: "rgba(75,159,255,0.45)", label: "#9ecbff" },
  4: { border: "#c46bff", glow: "rgba(196,107,255,0.45)", label: "#e0b3ff" },
  5: { border: "#ffc34b", glow: "rgba(255,195,75,0.5)", label: "#ffe09a" },
};
