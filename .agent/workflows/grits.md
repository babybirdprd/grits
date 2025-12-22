---
description: How to manage issues with Grits
---

# Grits Issue Management Workflow

Grits is a Git-native issue tracker. Use these tools and conventions when working with issues.

## Available MCP Tools

The Grits MCP server provides these tools:

1. **list_issues** - List all issues with optional filters
   - `status`: "open", "in-progress", "blocked", "closed"
   - `assignee`: username filter
   - `priority`: 1 (critical) to 5 (trivial)
   - `issue_type`: "bug", "feature", "task", "epic"

2. **create_issue** - Create a new issue
   - Requires: `title`
   - Optional: `description`, `issue_type` (default: "task"), `priority` (default: 2)

3. **update_issue** - Update an existing issue
   - Requires: `id`
   - Optional: `title`, `description`, `status`, `priority`, `assignee`

4. **close_issue** - Close an issue by ID

5. **get_issue** - Get full details of an issue by ID

## Best Practices

// turbo-all

1. Check existing issues before creating new ones:
   ```
   Use list_issues with appropriate filters
   ```

2. Use meaningful titles that describe the problem/feature, not the solution

3. Reference issue IDs in commit messages:
   ```
   feat: implement login flow (gr-a1b2)
   ```

4. Close issues with completion message:
   ```
   close_issue with id="gr-a1b2"
   ```

## Priority Guidelines

- **P1 (Critical)**: Production down, data loss, security issue
- **P2 (High)**: Major feature broken, blocking other work
- **P3 (Medium)**: Important but not urgent
- **P4 (Low)**: Nice to have, minor improvement
- **P5 (Trivial)**: Cosmetic, documentation

## Issue Types

- **bug**: Something is broken
- **feature**: New functionality
- **task**: Work item or chore
- **epic**: Large feature with sub-issues

## Kanban Column Mapping

| Status | Column | Meaning |
|--------|--------|---------|
| open | Open | Ready to start |
| in-progress | In Progress | Actively being worked on |
| blocked | Blocked | Waiting on external factor |
| closed | Closed | Done or won't fix |
