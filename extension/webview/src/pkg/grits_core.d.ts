/* tslint:disable */
/* eslint-disable */

export class WasmFileSystem {
  free(): void;
  [Symbol.dispose](): void;
  constructor();
}

export class WasmGit {
  free(): void;
  [Symbol.dispose](): void;
  constructor();
}

export class WasmStore {
  free(): void;
  [Symbol.dispose](): void;
  /**
   * Get vitals summary for dashboard
   */
  get_vitals(topology_json?: string | null): string;
  add_comment(id: string, text: string, author: string): void;
  bulk_update(ids_json: string, field: string, value_json: string): void;
  /**
   * List issues with optional filters (passed as JSON object)
   * { "status": "open", "sort_by": "updated" }
   */
  list_issues(filter_json?: string | null): string;
  create_issue(title: string, description: string, issue_type: string, priority: number): string;
  remove_label(id: string, label: string): void;
  update_issue(id: string, field: string, value_json: string): void;
  get_all_labels(): string;
  /**
   * Compute solid score from topology JSON
   */
  compute_solid_score(topology_json: string): number;
  /**
   * Get topology graph data for 3D visualization
   * Returns JSON with {nodes: {id: {name, kind, file_path, pageRank, inCycle}}, edges: [[src, dst, {relation, strength}]]}
   */
  get_topology_for_viz(topology_json: string): string;
  /**
   * Get PageRank hotspots (top N most connected symbols)
   */
  get_pagerank_hotspots(topology_json: string, limit: number): string;
  constructor();
  /**
   * Load issues from JSONL content string
   */
  load(content: string): void;
  /**
   * Export issues to JSONL content string
   */
  export(): string;
  search(query: string): string;
  add_label(id: string, label: string): void;
}

export class WasmTopologyStore {
  free(): void;
  [Symbol.dispose](): void;
  /**
   * Get edge count.
   */
  edge_count(): number;
  /**
   * Get node count.
   */
  node_count(): number;
  /**
   * Get top N symbols by PageRank (cached).
   */
  get_hotspots(limit: number): string;
  /**
   * Load topology from JSON and pre-compute PageRank + Solid Score.
   * Call this once on dashboard open for instant subsequent queries.
   */
  load_topology(topology_json: string): string;
  /**
   * Instant search for symbols by name (fuzzy match).
   * Returns in <10ms from pre-loaded graph.
   */
  search_symbols(query: string, limit: number): string;
  /**
   * Get cached solid score (instant, no computation).
   */
  get_solid_score(): number;
  /**
   * Get edges for visualization.
   */
  get_edges_for_viz(): string;
  /**
   * Get node data for 3D visualization (pre-computed layout data).
   */
  get_nodes_for_viz(): string;
  constructor();
  /**
   * Get cached Betti numbers as JSON.
   */
  get_betti(): string;
  /**
   * Check if topology is loaded.
   */
  is_loaded(): boolean;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_wasmfilesystem_free: (a: number, b: number) => void;
  readonly __wbg_wasmgit_free: (a: number, b: number) => void;
  readonly __wbg_wasmstore_free: (a: number, b: number) => void;
  readonly __wbg_wasmtopologystore_free: (a: number, b: number) => void;
  readonly wasmstore_add_comment: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number];
  readonly wasmstore_add_label: (a: number, b: number, c: number, d: number, e: number) => [number, number];
  readonly wasmstore_bulk_update: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number];
  readonly wasmstore_compute_solid_score: (a: number, b: number, c: number) => [number, number, number];
  readonly wasmstore_create_issue: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number, number, number];
  readonly wasmstore_export: (a: number) => [number, number, number, number];
  readonly wasmstore_get_all_labels: (a: number) => [number, number, number, number];
  readonly wasmstore_get_pagerank_hotspots: (a: number, b: number, c: number, d: number) => [number, number, number, number];
  readonly wasmstore_get_topology_for_viz: (a: number, b: number, c: number) => [number, number, number, number];
  readonly wasmstore_get_vitals: (a: number, b: number, c: number) => [number, number, number, number];
  readonly wasmstore_list_issues: (a: number, b: number, c: number) => [number, number, number, number];
  readonly wasmstore_load: (a: number, b: number, c: number) => [number, number];
  readonly wasmstore_new: () => number;
  readonly wasmstore_remove_label: (a: number, b: number, c: number, d: number, e: number) => [number, number];
  readonly wasmstore_search: (a: number, b: number, c: number) => [number, number, number, number];
  readonly wasmstore_update_issue: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number];
  readonly wasmtopologystore_edge_count: (a: number) => number;
  readonly wasmtopologystore_get_betti: (a: number) => [number, number];
  readonly wasmtopologystore_get_edges_for_viz: (a: number) => [number, number, number, number];
  readonly wasmtopologystore_get_hotspots: (a: number, b: number) => [number, number, number, number];
  readonly wasmtopologystore_get_nodes_for_viz: (a: number) => [number, number, number, number];
  readonly wasmtopologystore_get_solid_score: (a: number) => number;
  readonly wasmtopologystore_is_loaded: (a: number) => number;
  readonly wasmtopologystore_load_topology: (a: number, b: number, c: number) => [number, number, number, number];
  readonly wasmtopologystore_new: () => number;
  readonly wasmtopologystore_node_count: (a: number) => number;
  readonly wasmtopologystore_search_symbols: (a: number, b: number, c: number, d: number) => [number, number, number, number];
  readonly wasmfilesystem_new: () => number;
  readonly wasmgit_new: () => number;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_externrefs: WebAssembly.Table;
  readonly __externref_table_dealloc: (a: number) => void;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
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
