/* tslint:disable */
/* eslint-disable */

/**
 * WASM-facing match handle. Plugin and commands cross the boundary as JSON strings.
 */
export class WasmMatch {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Apply a player command (JSON). Returns serialized `Vec<Event>`.
     */
    apply(player_id: number, command_json: string): string;
    /**
     * Create a match from a plugin bundle JSON and seed.
     *
     * Expected plugin shape:
     * `{ "manifest": {...}, "units": [...], "traits": [...], "abilities": [...] }`
     */
    constructor(plugin_json: string, seed: bigint);
    /**
     * Full match snapshot as JSON (`MatchSnapshot`).
     */
    snapshot_json(): string;
    /**
     * Deterministic state hash as 16-char lowercase hex.
     */
    state_hash(): string;
}

export function wasm_engine_version(): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_wasmmatch_free: (a: number, b: number) => void;
    readonly wasm_engine_version: () => [number, number];
    readonly wasmmatch_apply: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly wasmmatch_new: (a: number, b: number, c: bigint) => [number, number, number];
    readonly wasmmatch_snapshot_json: (a: number) => [number, number, number, number];
    readonly wasmmatch_state_hash: (a: number) => [number, number];
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
