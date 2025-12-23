---
description: How to test the Grits MCP server in a new session
---

# MCP Server Testing Workflow

Complete verification of all 16 Grits MCP tools. Requires a **fresh session** after configuring MCP.

## Prerequisites

- `gr` CLI installed (`cargo install grits-cli`)
- Project initialized with `gr onboard`
- `.vscode/mcp.json` configured:
```json
{"servers":{"grits":{"command":"gr","args":["serve-mcp"]}}}
```

---

## Test Suites

### 1. CRUD Operations (5 tools)

| Prompt | Expected Tool | Purpose |
|--------|---------------|---------|
| "List all open issues" | `list_issues` | Filter by status, assignee, priority, type |
| "Create a bug called 'Login crash' with priority 1" | `create_issue` | Create with title, description, type, priority |
| "Show details of issue gr-xxx" | `get_issue` | Get full issue details |
| "Update gr-xxx status to in-progress" | `update_issue` | Update status, priority, assignee, title, description |
| "Close issue gr-xxx" | `close_issue` | Mark issue closed |

---

### 2. Strategic AI-Native Operations (6 tools)

| Prompt | Expected Tool | Purpose |
|--------|---------------|---------|
| "What should I work on next?" | `get_next_task` | Priority-weighted suggestion with reasoning |
| "Search for issues about authentication" | `search_issues` | BM25-ranked text search across title/description |
| "Find duplicate issues" | `detect_duplicates` | Jaccard similarity on titles |
| "Show the dependency graph" | `get_issue_graph` | DAG of all issues with edges |
| "Find stale issues older than 30 days" | `cleanup_stale` | List issues with no updates |
| "Summarize sprint progress" | `summarize_sprint` | Stats: total, by status, by priority, by type |

---

### 3. Contextual Operations (3 tools)

| Prompt | Expected Tool | Purpose |
|--------|---------------|---------|
| "Find issues related to gr-xxx" | `find_related_issues` | Keyword overlap matching |
| "What issue matches this error: [paste error]" | `suggest_issue_for_error` | Error message pattern matching |
| "What issues relate to this diff?" | `infer_issue_from_diff` | Parse diff for file paths, find mentions |

---

### 4. Bulk Operations (1 tool)

| Prompt | Expected Tool | Purpose |
|--------|---------------|---------|
| "Triage issues gr-a, gr-b, gr-c with status open, priority 2" | `bulk_triage` | Batch update multiple issues at once |

---

### 5. Workflow Assistance (2 tools)

| Prompt | Expected Tool | Purpose |
|--------|---------------|---------|
| "Link my last 5 commits to related issues" | `link_commit_to_issues` | Parse commit messages for `gr-xxx` refs |
| "Create issues from TODO comments in src/" | `generate_issue_from_todo` | Scan files for TODO/FIXME patterns |

---

## Verification Checklist

- [ ] All 16 tools are discoverable (agent can list them)
- [ ] CRUD: create → get → update → list → close cycle works
- [ ] Search: returns BM25-ranked relevant results
- [ ] Graph: returns nodes and edges for visualization
- [ ] Contextual: tools analyze provided context correctly
- [ ] Bulk: updates multiple issues in single call
- [ ] Workflow: integrates with git repository

## Troubleshooting

If tools aren't visible:
1. Verify `gr serve-mcp` runs without error: `gr serve-mcp`
2. Check MCP config path: `.vscode/mcp.json`
3. Restart VS Code / start new agent session
4. Check CLI version: `gr --version` (should be 1.0.0+)
