
export class WasmStore {
    free(): void;
    constructor();
    list_issues(filter_json: string): string;
    create_issue(issue_json: string): void;
    update_issue(issue_json: string): void;
    save_to_jsonl(): string;
    load_from_jsonl(content: string): void;
    load_workspace(contents: any[]): void;
    search(query: string): string;
    get_issue(id: string): string;
    add_comment(issue_id: string, author: string, text: string): string;
    add_label(issue_id: string, label: string): string;
    remove_label(issue_id: string, label: string): string;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly wasmstore_new: () => number;
  readonly wasmstore_load_from_jsonl: (a: number, b: number, c: number) => Array;
  readonly wasmstore_load_workspace: (a: number, b: number, c: number) => Array;
  readonly wasmstore_list_issues: (a: number, b: number, c: number) => Array;
  readonly wasmstore_get_issue: (a: number, b: number, c: number) => Array;
  readonly wasmstore_update_issue: (a: number, b: number, c: number) => Array;
  readonly wasmstore_create_issue: (a: number, b: number, c: number) => Array;
  readonly wasmstore_search: (a: number, b: number, c: number) => Array;
  readonly wasmstore_save_to_jsonl: (a: number) => Array;
  readonly wasmstore_add_comment: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => Array;
  readonly wasmstore_add_label: (a: number, b: number, c: number, d: number, e: number) => Array;
  readonly wasmstore_remove_label: (a: number, b: number, c: number, d: number, e: number) => Array;
  readonly __wbg_wasmstore_free: (a: number, b: number) => void;
}

export default function init(module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;
