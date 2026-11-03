declare namespace wasm_bindgen {
    /* tslint:disable */
    /* eslint-disable */

    export class Scrubbed {
        private constructor();
        free(): void;
        [Symbol.dispose](): void;
        audio(): Float32Array;
        hum_removed_hz(): number | undefined;
        ridges_notched(): number;
    }

    export function analyze(x: Float32Array | number[], sr: number): string;

    export function demo(sr: number): Float32Array;

    export function scrub(x: Float32Array | number[], sr: number, noise_dbfs: number): Scrubbed;

    /**
     * Returns [frames, bins, hop_s, bin_hz, lo_db, hi_db, ...pixels 0..=255 frame-major].
     * Pass the lo_db/hi_db of an earlier call to draw this clip on the same scale.
     */
    export function spectrogram(x: Float32Array | number[], sr: number, max_hz: number, lo_db?: number | null, hi_db?: number | null): Float32Array;

}
declare type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

declare interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_scrubbed_free: (a: number, b: number) => void;
    readonly analyze: (a: number, b: number, c: number) => [number, number];
    readonly demo: (a: number) => [number, number];
    readonly scrub: (a: number, b: number, c: number, d: number) => number;
    readonly scrubbed_audio: (a: number) => [number, number];
    readonly scrubbed_hum_removed_hz: (a: number) => number;
    readonly scrubbed_ridges_notched: (a: number) => number;
    readonly spectrogram: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number];
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
