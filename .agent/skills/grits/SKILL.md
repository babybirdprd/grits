---
name: Grits Issue Tracker
description: Manage tasks, track issues, and retrieve graph-based code context using the Grits CLI.
version: 1.0.0
---

# Grits Issue Tracker

Grits (`gr`) is a local-first, git-native issue tracker designed for both humans and agents. It uses a topological graph of the codebase to provide context-aware task management.

## Capabilities

*   **Task Management**: Create, list, update, and close issues without leaving the terminal.
*   **Context Awareness**: Retrieve "Star Neighborhoods" (connected files) to understand code dependencies.
*   **Focus Tracking**: Set a "Sticky Focus" (`gr workon`) to persist the current task context across sessions.
*   **Context Assembly**: Bundle relevant code snippets into a markdown format for analysis.

## Usage

### 1. Check Project Pulse

Use `gr pulse` to hydrate your session. It returns the current focus, blocked issues, and recent git activity.

```bash
gr pulse
```

### 2. Start Working on an Issue

Sets the "Sticky Focus" to a specific issue. This updates the issue status to `in-progress` and outputs the issue description along with connected files.

```bash
gr workon <issue-id>
```

### 3. Retrieve Code Context (Star Neighborhood)

Finds files that are topologically connected (imports/calls) to a given file. Use this to determine which files to edit when fixing a bug or adding a feature.

```bash
gr star <file-path>
```

### 4. Create a New Issue

```bash
gr create "Title of the task" --type feature --priority 2 --description "Detailed description..."
```

### 5. Assemble Context Bundle

Generates a focused "Mini Codebase" containing only the symbols and code paths relevant to a specific issue or set of seed symbols.

```bash
gr context assemble --issue <issue-id> --format markdown
```

### 6. Update Issue Status

```bash
# Update status via short-hand
gr update --id <issue-id> --status closed

# Or use the set command for quick updates
gr set --id <issue-id> status:closed
```
