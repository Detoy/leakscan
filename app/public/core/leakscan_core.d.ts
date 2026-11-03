declare namespace wasm_bindgen {
    /* tslint:disable */
    /* eslint-disable */

    export function analyze(x: Float32Array | number[], sr: number): string;

    export function demo(sr: number): Float32Array;

    export function scrub(x: Float32Array | number[], sr: number, noise_dbfs: number): Float32Array;

    /**
     * Returns [frames, bins, hop_s, bin_hz, ...u8 dB image row-major by frame].
     */
    export function spectrogram(x: Float32Array | number[], sr: number, max_hz: number): Float32Array;

}
declare type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

declare interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly analyze: (a: number, b: number, c: number) => [number, number];
    readonly demo: (a: number) => [number, number];
    readonly scrub: (a: number, b: number, c: number, d: number) => [number, number];
    readonly spectrogram: (a: number, b: number, c: number, d: number) => [number, number];
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_start: () => void;
}

declare type SyncInitInput = BufferSource | WebAssembly.Module;

declare namespace wasm_bindgen {
    /**
     * Instantiates the given `module`, which can either be bytes or
     * a precompiled `WebAssembly.Module`.
     *
     * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
     *
     * @returns {InitOutput}
     */
    export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;
}

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
declare function wasm_bindgen (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
