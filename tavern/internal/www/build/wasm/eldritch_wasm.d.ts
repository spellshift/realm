/* tslint:disable */
/* eslint-disable */

export class ExecutionResult {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    readonly clear: boolean;
    readonly echo: string | undefined;
    readonly output: string | undefined;
}

export class HeadlessRepl {
    free(): void;
    [Symbol.dispose](): void;
    complete(line: string, cursor: number): string;
    input(line: string): string;
    constructor();
    reset(): void;
}

export class RenderState {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    readonly buffer: string;
    readonly cursor: number;
    readonly prompt: string;
    readonly suggestion_idx: number | undefined;
    readonly suggestions: any[] | undefined;
}

export class WasmRepl {
    free(): void;
    [Symbol.dispose](): void;
    get_history(): any[];
    get_state(): RenderState;
    handle_key(key: string, ctrl: boolean, _alt: boolean, meta: boolean, shift: boolean): ExecutionResult;
    handle_paste(text: string): ExecutionResult;
    load_history(js_history: any[]): void;
    constructor();
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_executionresult_free: (a: number, b: number) => void;
    readonly __wbg_headlessrepl_free: (a: number, b: number) => void;
    readonly __wbg_renderstate_free: (a: number, b: number) => void;
    readonly __wbg_wasmrepl_free: (a: number, b: number) => void;
    readonly executionresult_clear: (a: number) => number;
    readonly executionresult_echo: (a: number, b: number) => void;
    readonly executionresult_output: (a: number, b: number) => void;
    readonly headlessrepl_complete: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly headlessrepl_input: (a: number, b: number, c: number, d: number) => void;
    readonly headlessrepl_new: () => number;
    readonly headlessrepl_reset: (a: number) => void;
    readonly renderstate_buffer: (a: number, b: number) => void;
    readonly renderstate_cursor: (a: number) => number;
    readonly renderstate_prompt: (a: number, b: number) => void;
    readonly renderstate_suggestion_idx: (a: number) => number;
    readonly renderstate_suggestions: (a: number, b: number) => void;
    readonly wasmrepl_get_history: (a: number, b: number) => void;
    readonly wasmrepl_get_state: (a: number) => number;
    readonly wasmrepl_handle_key: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => number;
    readonly wasmrepl_handle_paste: (a: number, b: number, c: number) => number;
    readonly wasmrepl_load_history: (a: number, b: number, c: number) => void;
    readonly wasmrepl_new: () => number;
    readonly __wbindgen_export: (a: number, b: number) => number;
    readonly __wbindgen_export2: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
    readonly __wbindgen_export3: (a: number, b: number, c: number) => void;
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
