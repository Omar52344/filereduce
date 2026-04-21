/* tslint:disable */
/* eslint-disable */

export class FileReduceWasm {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Compress JSONL data to .fra format (synchronous)
     */
    compress_to_fra(jsonl_bytes: Uint8Array): Uint8Array;
    /**
     * Convert EDIFACT text to JSONL string (synchronous)
     */
    convert_edi_to_jsonl(edi_text: string): string;
    /**
     * Convert EDIFACT to JSONL asynchronously (for large files)
     */
    convert_edi_to_jsonl_async(edi_bytes: Uint8Array): Promise<any>;
    /**
     * Convert EDIFACT bytes to JSONL bytes (synchronous)
     */
    convert_edi_to_jsonl_bytes(edi_bytes: Uint8Array): Uint8Array;
    /**
     * Convert JSONL to EDIFACT (placeholder - not yet implemented)
     */
    convert_jsonl_to_edi(_jsonl_bytes: Uint8Array): string;
    /**
     * Decompress .fra data to JSONL (synchronous)
     */
    decompress_from_fra(fra_bytes: Uint8Array): Uint8Array;
    constructor();
}

export function compress_jsonl_simple(jsonl_bytes: Uint8Array): Uint8Array;

export function convert_edi_to_jsonl_simple(edi_text: string): string;

export function decompress_fra_simple(fra_bytes: Uint8Array): Uint8Array;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_filereducewasm_free: (a: number, b: number) => void;
    readonly compress_jsonl_simple: (a: number, b: number) => [number, number, number, number];
    readonly convert_edi_to_jsonl_simple: (a: number, b: number) => [number, number, number, number];
    readonly decompress_fra_simple: (a: number, b: number) => [number, number, number, number];
    readonly filereducewasm_compress_to_fra: (a: number, b: number, c: number) => [number, number, number, number];
    readonly filereducewasm_convert_edi_to_jsonl: (a: number, b: number, c: number) => [number, number, number, number];
    readonly filereducewasm_convert_edi_to_jsonl_async: (a: number, b: number, c: number) => any;
    readonly filereducewasm_convert_edi_to_jsonl_bytes: (a: number, b: number, c: number) => [number, number, number, number];
    readonly filereducewasm_convert_jsonl_to_edi: (a: number, b: number, c: number) => [number, number, number, number];
    readonly filereducewasm_decompress_from_fra: (a: number, b: number, c: number) => [number, number, number, number];
    readonly filereducewasm_new: () => number;
    readonly rust_zstd_wasm_shim_calloc: (a: number, b: number) => number;
    readonly rust_zstd_wasm_shim_free: (a: number) => void;
    readonly rust_zstd_wasm_shim_malloc: (a: number) => number;
    readonly rust_zstd_wasm_shim_memcmp: (a: number, b: number, c: number) => number;
    readonly rust_zstd_wasm_shim_memcpy: (a: number, b: number, c: number) => number;
    readonly rust_zstd_wasm_shim_memmove: (a: number, b: number, c: number) => number;
    readonly rust_zstd_wasm_shim_memset: (a: number, b: number, c: number) => number;
    readonly rust_zstd_wasm_shim_qsort: (a: number, b: number, c: number, d: number) => void;
    readonly wasm_bindgen__convert__closures_____invoke__h8cccdd06361b3129: (a: number, b: number, c: any) => [number, number];
    readonly wasm_bindgen__convert__closures_____invoke__h4a62e39e18fd861d: (a: number, b: number, c: any, d: any) => void;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_destroy_closure: (a: number, b: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
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
