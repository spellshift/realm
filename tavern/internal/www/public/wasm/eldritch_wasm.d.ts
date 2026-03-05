/* tslint:disable */
/* eslint-disable */

export class BrowserRepl {
    free(): void;
    [Symbol.dispose](): void;
    complete(line: string, cursor: number): string;
    get_suggestions(): Array<any> | undefined;
    get_suggestions_index(): number | undefined;
    get_suggestions_start(): number | undefined;
    handle_key(key: string, ctrl: boolean, alt: boolean, meta: boolean, shift: boolean): ReplState;
    input(line: string): string;
    constructor();
    reset(): void;
}

export class ExecutionResult {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    readonly clear: boolean;
    readonly echo: string | undefined;
    readonly output: string | undefined;
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

export class ReplState {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    get args(): string[] | undefined;
    set args(value: string[] | null | undefined);
    buffer: string;
    cursor_pos: number;
    get function(): string | undefined;
    set function(value: string | null | undefined);
    is_running: boolean;
    get payload(): string | undefined;
    set payload(value: string | null | undefined);
    prompt: string;
    status: string;
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

export class WasmSsh {
    free(): void;
    [Symbol.dispose](): void;
    constructor(target_user: string, on_send: Function, on_stdout: Function, on_stderr: Function, on_disconnect: Function, cols: number, rows: number);
    on_stdin(data: Uint8Array): void;
    on_tcp_recv(data: Uint8Array): void;
    resize_pty(cols: number, rows: number): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_browserrepl_free: (a: number, b: number) => void;
    readonly __wbg_executionresult_free: (a: number, b: number) => void;
    readonly __wbg_get_replstate_args: (a: number, b: number) => void;
    readonly __wbg_get_replstate_buffer: (a: number, b: number) => void;
    readonly __wbg_get_replstate_cursor_pos: (a: number) => number;
    readonly __wbg_get_replstate_function: (a: number, b: number) => void;
    readonly __wbg_get_replstate_is_running: (a: number) => number;
    readonly __wbg_get_replstate_payload: (a: number, b: number) => void;
    readonly __wbg_get_replstate_prompt: (a: number, b: number) => void;
    readonly __wbg_get_replstate_status: (a: number, b: number) => void;
    readonly __wbg_renderstate_free: (a: number, b: number) => void;
    readonly __wbg_replstate_free: (a: number, b: number) => void;
    readonly __wbg_set_replstate_args: (a: number, b: number, c: number) => void;
    readonly __wbg_set_replstate_buffer: (a: number, b: number, c: number) => void;
    readonly __wbg_set_replstate_cursor_pos: (a: number, b: number) => void;
    readonly __wbg_set_replstate_function: (a: number, b: number, c: number) => void;
    readonly __wbg_set_replstate_is_running: (a: number, b: number) => void;
    readonly __wbg_set_replstate_payload: (a: number, b: number, c: number) => void;
    readonly __wbg_set_replstate_prompt: (a: number, b: number, c: number) => void;
    readonly __wbg_set_replstate_status: (a: number, b: number, c: number) => void;
    readonly __wbg_wasmrepl_free: (a: number, b: number) => void;
    readonly __wbg_wasmssh_free: (a: number, b: number) => void;
    readonly browserrepl_complete: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly browserrepl_get_suggestions: (a: number) => number;
    readonly browserrepl_get_suggestions_index: (a: number) => number;
    readonly browserrepl_get_suggestions_start: (a: number) => number;
    readonly browserrepl_handle_key: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => number;
    readonly browserrepl_input: (a: number, b: number, c: number, d: number) => void;
    readonly browserrepl_new: () => number;
    readonly browserrepl_reset: (a: number) => void;
    readonly executionresult_clear: (a: number) => number;
    readonly executionresult_echo: (a: number, b: number) => void;
    readonly executionresult_output: (a: number, b: number) => void;
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
    readonly wasmssh_new: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => number;
    readonly wasmssh_on_stdin: (a: number, b: number, c: number) => void;
    readonly wasmssh_on_tcp_recv: (a: number, b: number, c: number) => void;
    readonly wasmssh_resize_pty: (a: number, b: number, c: number) => void;
    readonly __wasm_bindgen_func_elem_2047: (a: number, b: number) => void;
    readonly __wasm_bindgen_func_elem_4289: (a: number, b: number, c: number, d: number) => void;
    readonly __wasm_bindgen_func_elem_4306: (a: number, b: number, c: number, d: number) => void;
    readonly __wbindgen_export: (a: number, b: number) => number;
    readonly __wbindgen_export2: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_export3: (a: number) => void;
    readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
    readonly __wbindgen_export4: (a: number, b: number, c: number) => void;
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
