/* tslint:disable */
/* eslint-disable */

/**
 * The positions of a text in an editor: a line and a column in UTF-16, both
 * from 0.
 */
export class TextPositions {
    free(): void;
    [Symbol.dispose](): void;
    constructor(text: string);
    /**
     * The offset at `line` and the column in UTF-16 `column`. A column past the
     * end of its line is the end of the line, and a line past the last line
     * is the end of the text.
     */
    offset(line: number, column: number): number;
    /**
     * The line and the column in UTF-16 of the offset `offset`. An offset past
     * the end of the text is the end of the text. After a final newline, the
     * end of the text is at the start of the line after it, where an editor
     * shows it.
     */
    position(offset: number): Uint32Array;
    /**
     * The line and the UTF-16 column of each offset in `offsets`
     * (`position`), flattened to `[line, column, line, column, …]`. One call
     * converts all the offsets, instead of one call across the WebAssembly
     * boundary per offset.
     */
    positions(offsets: Uint32Array | number[]): Uint32Array;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_textpositions_free: (a: number, b: number) => void;
    readonly textpositions_new: (a: number, b: number) => number;
    readonly textpositions_offset: (a: number, b: number, c: number) => number;
    readonly textpositions_position: (a: number, b: number) => [number, number];
    readonly textpositions_positions: (a: number, b: number, c: number) => [number, number];
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
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
