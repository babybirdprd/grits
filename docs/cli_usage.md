# CLI Usage

The `gr` CLI is the primary interface for interacting with Grits issues.

## Common Commands

### `create`
Create a new issue.

```bash
# Create a bug with default priority
gr create "Fix login crash"

# Create a feature request with specific priority and description
gr create "Dark Mode" --type feature --priority 1 --description "Add dark mode support"
```

### `list`
List issues. Supports filtering and sorting.

```bash
# List all open issues
gr list

# Filter by status
gr list --status in-progress

# Filter by assignee
gr list --assignee "jane.doe"

# Filter by label
gr list --label "frontend"

# Sort by priority
gr list --sort priority
```

### `show`
Show details of a specific issue.

```bash
gr show <issue-id>
```

### `edit`
Edit an issue's description and metadata in your `$EDITOR`.

```bash
gr edit <issue-id>
```

### `update`
Update specific fields of an issue directly.

```bash
# Change status
gr update <issue-id> --status done

# Add a label
gr update <issue-id> --add-label "urgent"

# Add a blocking dependency
gr update <issue-id> --add-dependency <blocking-issue-id>
```

### `sync`
Synchronize local changes with the git backend. This exports DB changes to JSONL, commits, pulls, merges, and pushes.

```bash
gr sync
```

### `config`
Manage configuration values.

```bash
# Set user name
gr config set user.name "Alice"

# List all config
gr config list
```

### `stats`
Show issue statistics.

```bash
gr stats
```

### `onboard`
Initialize a new Grits repository in the current directory.

```bash
gr onboard
```

## Global Options

*   `-h, --help`: Print help information.
