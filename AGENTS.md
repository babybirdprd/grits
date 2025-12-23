# Grits Strategic CLI for AI Agents

Grits provides a set of high-level "Strategic Tools" via the `gr` CLI, specifically designed to help AI agents manage complex projects, analyze issue graphs, and automate workflows.

## Global Options

- `--root <PATH>`: Explicitly set the project root directory. This ensures the CLI and MCP server always use the same `grits.db`.

## Strategic Command Groups

### 1. Advisory (`gr advisory`)
High-level guidance on what to do next.

- `gr advisory next`: Suggest the next actionable task based on priority, blockers, and recent activity.
  - `--assignee <NAME>`: Filter for a specific person.
- `gr advisory sprint`: Summarize progress over the last week.
  - `--days <N>`: Change the lookback period (default: 7).

### 2. Analysis (`gr analysis`)
Deep insights into the issue graph and repository.

- `gr analysis graph`: Output a JSON dependency graph of all issues.
- `gr analysis duplicates`: Detect potential duplicate issues based on title similarity.
- `gr analysis related`: Find issues mentioning a specific file.
  - `<FILE>`: Path to the file.
- `gr analysis search`: BM25-ranked natural language search.
  - `<QUERY>`: Search terms.
  - `--limit <N>`: Max results.

### 3. Workflow (`gr workflow`)
Automation and batch operations.

- `gr workflow triage`: Batch update status/priority/assignee for multiple issues.
  - `--ids <ID1,ID2...>`
  - `--status <STATUS>`
  - `--priority <1-5>`
- `gr workflow stale`: Identify issues that haven't been updated in 30 days.

### 4. Context (`gr context`)
Bridge the gap between code and issues.

- `gr context error`: Match a compiler error message against existing issues.
  - `<ERROR>`: The error message.
- `gr context diff`: Analyze current Git changes and suggest a title/description for a new issue.
- `gr context todo`: Scan files for `TODO` comments and suggest issues.
  - `<FILE>`: File to scan.
- `gr context commit`: Parse a commit message for issue references (fixes #123, etc).

## Usage for Agents

When you are unsure what to work on, start with `gr advisory next`. If you encounter an error, use `gr context error` to see if it's a known issue. Before creating a new issue, use `gr analysis duplicates` to avoid redundancy.
