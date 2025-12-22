# CLI Usage

The `bd` CLI is the primary interface for interacting with Beads issues.

## Common Commands

### `create`
Create a new issue.

```bash
# Create a bug with default priority
bd create "Fix login crash"

# Create a feature request with specific priority and description
bd create "Dark Mode" --type feature --priority 1 --description "Add dark mode support"
```

### `list`
List issues. Supports filtering and sorting.

```bash
# List all open issues
bd list

# Filter by status
bd list --status in-progress

# Filter by assignee
bd list --assignee "jane.doe"

# Filter by label
bd list --label "frontend"

# Sort by priority
bd list --sort priority
```

### `show`
Show details of a specific issue.

```bash
bd show <issue-id>
```

### `edit`
Edit an issue's description and metadata in your `$EDITOR`.

```bash
bd edit <issue-id>
```

### `update`
Update specific fields of an issue directly.

```bash
# Change status
bd update <issue-id> --status done

# Add a label
bd update <issue-id> --add-label "urgent"

# Add a blocking dependency
bd update <issue-id> --add-dependency <blocking-issue-id>
```

### `sync`
Synchronize local changes with the git backend. This exports DB changes to JSONL, commits, pulls, merges, and pushes.

```bash
bd sync
```

### `config`
Manage configuration values.

```bash
# Set user name
bd config set user.name "Alice"

# List all config
bd config list
```

### `stats`
Show issue statistics.

```bash
bd stats
```

### `onboard`
Initialize a new Beads repository in the current directory.

```bash
bd onboard
```

## Global Options

*   `-h, --help`: Print help information.
