/* tslint:disable */
/* eslint-disable */
/**
* Initialize the front-end library
* @param {number} port
*/
export function init_usdpl(port: number): void;
/**
* Get the targeted plugin framework, or "any" if unknown
* @returns {string}
*/
export function target(): string;
/**
* Call a function on the back-end.
* Returns null (None) if this fails for any reason.
* @param {string} name
* @param {any[]} parameters
* @returns {Promise<any>}
*/
export function call_backend(name: string, parameters: any[]): Promise<any>;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly init_usdpl: (a: number) => void;
  readonly target: (a: number) => void;
  readonly call_backend: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_export_0: (a: number) => number;
  readonly __wbindgen_export_1: (a: number, b: number, c: number) => number;
  readonly __wbindgen_export_2: WebAssembly.Table;
  readonly __wbindgen_export_3: (a: number, b: number, c: number) => void;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_export_4: (a: number, b: number) => void;
  readonly __wbindgen_export_5: (a: number) => void;
  readonly __wbindgen_export_6: (a: number, b: number, c: number, d: number) => void;
}

/**
* Synchronously compiles the given `bytes` and instantiates the WebAssembly module.
*
* @param {BufferSource} bytes
*
* @returns {InitOutput}
*/
export function initSync(bytes: BufferSource): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
export default function init (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;


// USDPL customization
export function init_embedded();
