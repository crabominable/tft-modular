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

export type PluginBundle = {
  manifest: {
    id: string;
    name: string;
    version: string;
    api_version: string;
    description?: string;
  };
  units: Array<{
    id: string;
    name: string;
    cost: number;
    traits: string[];
    ability_id?: string | null;
    stats: Record<string, number>;
  }>;
  traits: unknown[];
  abilities: unknown[];
  modHash?: string;
};
