/* tslint:disable */
/* eslint-disable */

export class PseudocodeEngine {
  free(): void;
  [Symbol.dispose](): void;
  /**
   * Check syntax without executing
   */
  check_syntax(code: string): any;
  /**
   * Clear the input queue
   */
  clear_inputs(): void;
  /**
   * Get autocomplete suggestions at a given position
   */
  get_completions(code: string, line: number, column: number): any;
  /**
   * Get a virtual file from the file system
   */
  get_virtual_file(filename: string): string | undefined;
  /**
   * Set a virtual file in the file system
   */
  set_virtual_file(filename: string, content: string): void;
  /**
   * Check if there are more statements to execute
   */
  has_more_statements(): boolean;
  /**
   * Parse code and prepare for step-by-step execution
   */
  parse_for_execution(code: string): any;
  /**
   * Get all INPUT statements from code (variable names in order)
   */
  get_input_statements(code: string): any;
  /**
   * Execute the next statement and return output since last call
   */
  execute_next_statement(): any;
  /**
   * Get information about the next statement to execute
   */
  get_next_statement_info(): any;
  /**
   * Validate if a variable can be used for INPUT (before prompting)
   * Returns empty string if valid, or error message if invalid
   */
  validate_input_variable(var_name: string): string;
  constructor();
  /**
   * Execute pseudocode and return results
   */
  execute(code: string): any;
  /**
   * Add input to the input queue
   */
  add_input(input: string): void;
  /**
   * Get hover information at a given position
   */
  get_hover(code: string, line: number, column: number): any;
}

export function init(): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_pseudocodeengine_free: (a: number, b: number) => void;
  readonly init: () => void;
  readonly pseudocodeengine_add_input: (a: number, b: number, c: number) => void;
  readonly pseudocodeengine_check_syntax: (a: number, b: number, c: number) => any;
  readonly pseudocodeengine_clear_inputs: (a: number) => void;
  readonly pseudocodeengine_execute: (a: number, b: number, c: number) => any;
  readonly pseudocodeengine_execute_next_statement: (a: number) => any;
  readonly pseudocodeengine_get_completions: (a: number, b: number, c: number, d: number, e: number) => any;
  readonly pseudocodeengine_get_hover: (a: number, b: number, c: number, d: number, e: number) => any;
  readonly pseudocodeengine_get_input_statements: (a: number, b: number, c: number) => any;
  readonly pseudocodeengine_get_next_statement_info: (a: number) => any;
  readonly pseudocodeengine_get_virtual_file: (a: number, b: number, c: number) => [number, number];
  readonly pseudocodeengine_has_more_statements: (a: number) => number;
  readonly pseudocodeengine_new: () => number;
  readonly pseudocodeengine_parse_for_execution: (a: number, b: number, c: number) => any;
  readonly pseudocodeengine_set_virtual_file: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly pseudocodeengine_validate_input_variable: (a: number, b: number, c: number) => [number, number];
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_externrefs: WebAssembly.Table;
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
