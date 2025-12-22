export default async function init(url) { return Promise.resolve(); }
export class WasmStore {
    constructor() { this.issues = []; }
    load(content) {
        this.issues = content.split("\n").filter(l=>l.trim()).map(JSON.parse);
    }
    export() { return this.issues.map(JSON.stringify).join("\n"); }
    list_issues() { return JSON.stringify(this.issues); }
    update_issue() {}
    bulk_update() {}
    add_label() {}
    remove_label() {}
    add_comment() {}
    create_issue() { return ""; }
    get_all_labels() { return "[]"; }
    search() { return "[]"; }
}
