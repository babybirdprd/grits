---
description: Evaluate the Grits experience from an AI agent perspective
---

# Agent Experience Stress Test

This workflow creates an **intensive hands-on evaluation** of Grits. You will complete multiple real tasks using every major Grits feature, then analyze your experience.

> [!IMPORTANT]
> **Do NOT create a new Git branch for this evaluation.** All work happens on the current branch using Grits' issue tracking.

---

## Phase 1: Setup (3 min)

> [!IMPORTANT]
> **Read [AGENTS.md] FIRST** before running any commands. It contains critical context about Grits workflows, command shortcuts, and the "Golden Rules" for agents.

### 1.1 Fresh Start
```bash
# Verify OR initialize Grits
gr pulse || gr onboard --non-interactive

# Rebuild topology from scratch (watch the progress!)
gr analysis rebuild
```

### 1.2 Create Evaluation Tracking Issue
```bash
# Create the master tracking issue
gr create "Agent Stress Test - $(date +%Y-%m-%d)" \
  -d "Comprehensive Grits stress test following /agent-qa workflow" \
  -t epic -p 2

# Lock focus
gr workon <issue-id>
```

---

## Phase 2: The Gauntlet (30-45 min)

**You MUST complete ALL of these tasks.** Document friction points as you go.

### 2.1 Full Tour of Analysis Commands (10 min)

Run EVERY analysis command below. For each, note if it worked as expected.

```bash
# Star neighborhood - try with a real file
gr analysis star "grits-core/src/lib.rs" --depth 2

# Star neighborhood - try with a symbol  
gr analysis star "grits-core/src/lib.rs" --symbol "SqliteStore"

# Volumes - with file
gr analysis volumes "grits-core/src/topology/parser.rs"

# Volumes - from cache (no file arg)
gr analysis volumes

# Hotspots - who are the big players?
gr analysis hotspots --limit 10

# Hotspots - JSON output
gr analysis hotspots --format json --limit 5

# Cycle detection
gr refactor

# Layer check (may not have config - that's ok, note the error)
gr analysis check-layers --all

# Search for something
gr analysis search "dependency"

# Topology diff (shows changes since last rebuild)
gr analysis diff

# Trace shortest path between two symbols
gr analysis path "grits-core/src/lib.rs" "grits-core/src/topology/mod.rs"
```

Update your issue: `gr update --notes "Analysis tour complete. [friction notes here]"`

### 2.2 Issue CRUD Gauntlet (10 min)

Create, modify, link, and close issues rapidly.

```bash
# Create 3 child issues under your evaluation
gr create "Child A: Test create flow" -t task -p 3
gr create "Child B: Test update flow" -t bug -p 2  
gr create "Child C: Test dependency flow" -t feature -p 4

# Update one with various flags
gr update --id <child-a-id> --status in-progress --priority 1 --add-label "stress-test"

# Add a symbol to an issue
gr update --id <child-b-id> --add-symbol "grits-core/src/wasm.rs"

# Create a dependency chain: C depends on B, B depends on A
gr update --id <child-b-id> --add-dependency <child-a-id>
gr update --id <child-c-id> --add-dependency <child-b-id>

# Use the shorthand 'set' command (fuzzy)
gr set --id <child-a-id> stat:closed

# Check ready issues - should show C is blocked
gr ready

# Quick triage
gr workflow triage <child-b-id> <child-c-id> --status closed

# Attach a long-term memo to a symbol
gr memo attach "grits-core/src/lib.rs" "Main entry point - fragile"
```

### 2.3 Context Loading Challenge (10 min)

Use context tools to understand unfamiliar code.

```bash
# Pick a file you've never read before
# Try to understand it using ONLY Grits commands

# First: Get its star neighborhood
gr analysis star "grits-core/src/context.rs" --depth 1

# Second: Use context-bundle on your main issue
gr context-bundle <your-evaluation-issue-id>

# Third: Use context assemble (verify hydrated code snippets in output)
gr context assemble --symbols "grits-core/src/context.rs" --depth 1

# Fourth: Inspect your issue
gr inspect <your-evaluation-issue-id>

# Fifth: Check pulse
gr pulse
```

**Write a 2-sentence summary of what `context.rs` does**, using only info from Grits commands. Update: `gr update --notes "Context.rs does: ..."`

### 2.4 Edge Cases & Error Handling (5 min)

Deliberately try things that might break.

```bash
# Invalid issue ID
gr show nonexistent-id

# Missing topology cache (if you haven't rebuilt)
rm .grits/topology.json 2>/dev/null
gr analysis volumes

# Rebuild to restore
gr analysis rebuild

# Empty search
gr analysis search ""

# Star on unsupported file type
gr analysis star "Cargo.toml"

# Update with no ID and no focus set
gr workon --clear 2>/dev/null  # clear focus if this exists
gr update --status closed  # should error or use focus
```

Note which errors are helpful vs confusing.

---

## Phase 2.5: Problem-Solving Challenges (15 min)

**These challenges test if YOU can use Grits to solve real problems, not just run commands.**

### Challenge A: 🔍 Blind Navigation

**Goal**: Find where "issue storage" logic lives WITHOUT being told the file path.

Rules:
- You may NOT use `find`, `grep`, or read files directly
- You MUST use Grits commands to discover the location

Suggested approach:
```bash
# Start from hotspots - storage is probably central
gr analysis hotspots --limit 15

# Pick a promising candidate and explore its neighborhood
gr analysis star "<promising-file>" --depth 2

# Find tightly coupled clusters
gr analysis volumes
```

**Record your answer**: Which file(s) handle issue storage? How did Grits help?

Update: `gr update --notes "Blind Nav: Found storage at [X] using [commands]"`

### Challenge B: 🏗️ Blast Radius Assessment

**Scenario**: Someone wants to refactor `grits-core/src/models.rs`. What might break?

```bash
# Get the star neighborhood (who depends on models?)
gr analysis star "grits-core/src/models.rs" --depth 2

# Trace paths to key consumers
gr analysis path "grits-core/src/models.rs" "grits-cli/src/main.rs"

# Check if it's in any detected volumes
gr analysis volumes
```

**Record your answer**: List 3+ files that would be affected by changes to `models.rs`.

### Challenge C: 🐛 Root Cause Hunt

**Scenario**: You see this error: `"Failed to update issue: database locked"`

Use Grits to trace where database operations happen:
```bash
# Search issues for similar problems
gr issue search "database"
gr issue search "sqlite"

# Find the database-related hotspots
gr analysis hotspots --limit 20 --format json | grep -i store

# Trace the call path
gr analysis path "store.rs" "main.rs"
```

**Record your answer**: Which file would you investigate first? Why?

### Challenge D: ⚡ Speed Run (Timed)

**Test real handoff speed.**

1. Run `gr workon --clear` to reset focus
2. Start a timer
3. Run ONLY these commands:
   - `gr pulse`
   - `gr inspect <suggested-issue>`
   - `gr context-bundle <suggested-issue>`
4. Stop timer when you feel "ready to code"

**Record**: How many seconds? What was missing (if anything)?

---

## Phase 3: Simulate Session Handoff (5 min)

**Test the handoff scenario from scratch.**

1. Pretend you are a NEW agent with zero context
2. Run ONLY `gr pulse`
3. Can you figure out what's in-progress?
4. Run `gr inspect <suggested-issue>` 
5. Run `gr context-bundle <suggested-issue>`
6. Is there enough info to continue the work?

---

## Phase 4: Metrics Collection

### 4.1 Capture Final Topology State
```bash
gr analysis rebuild  # Fresh scan
gr pulse  # Get Solid Score

# Record these values:
gr analysis hotspots --limit 3
```

### 4.2 Fill in the Metrics Table

| Metric | Value | Notes |
|--------|-------|-------|
| Solid Score | ___ % | |
| Betti₀ (components) | ___ | |
| Betti₁ (cycles) | ___ | |
| Betti₂ (voids) | ___ | |
| Triangles | ___ | |
| Total Nodes | ___ | |
| Total Edges | ___ | |
| Top 3 Hotspots | 1. ___ 2. ___ 3. ___ | |
| Issues Created | ___ | |
| Commands Run | ~___ | |

---

## Phase 5: Report Generation

Create `agent_experience_qa_v<VERSION>.md` with this structure - save to repo:

```markdown
# Deep Agent Experience Evaluation: Grits v2.6.0

**Date**: YYYY-MM-DD
**Tracking Issue**: <issue-id>
**Commands Run**: ~N

---

## 1. Task Completed
[Summary of the gauntlet - which parts you completed]

## 2. Friction Points
| # | Moment | What Happened | Resolution | Suggested Fix |
|---|--------|---------------|------------|---------------|
| 1 | | | | |
| 2 | | | | |
(Continue for ALL friction points)

## 3. Error Message Audit
| Command | Error | Was it Helpful? | Suggestion |
|---------|-------|-----------------|------------|
| | | Yes/No/Partial | |

## 4. Topological Insights
[Metrics table + interpretation. Did geometry help?]

## 5. Problem-Solving Challenge Results
| Challenge | Result | What Worked | What Didn't |
|-----------|--------|-------------|-------------|
| Blind Navigation | Found: ___ | | |
| Blast Radius | Files affected: ___ | | |
| Root Cause Hunt | Investigate: ___ | | |
| Speed Run | Time: ___s | | |

## 6. Workflow Ergonomics
[AGENTS.md assessment, CLI usability, flags, shortcuts]

## 7. Session Handoff
[Could a fresh agent pick up your work? What's missing?]

## 8. Superpower Suggestion
[ONE feature that would make Grits 10x more valuable]

## 9. Final Verdict
**Indispensability Score: X/10**
[One-sentence summary]

## Evaluation Checklist
- [ ] Ran ALL analysis commands
- [ ] Completed issue CRUD gauntlet
- [ ] Tested context loading
- [ ] Tested error cases
- [ ] **Completed Blind Navigation challenge**
- [ ] **Completed Blast Radius challenge**
- [ ] **Completed Root Cause Hunt challenge**
- [ ] **Completed Speed Run (recorded time)**
- [ ] Simulated handoff
- [ ] Captured all metrics
- [ ] Created this report
```

---

## Phase 6: Cleanup

```bash
# Update your tracking issue with the report
gr update --description "[Summary of findings - see agent_experience_qa_v*.md]"
gr update --status closed

# Export for version control
gr export
```

---

## Success Criteria

You have successfully stress-tested Grits if:

1. ✅ Ran **every command** in the Analysis Tour
2. ✅ Created, updated, and linked **at least 3 issues**
3. ✅ Found **at least 3 friction points**
4. ✅ Identified **at least 1 unhelpful error message**
5. ✅ **Completed ALL 4 problem-solving challenges**
6. ✅ **Recorded Speed Run time** (target: <60 seconds to "ready to code")
7. ✅ Able to answer: "What is the Solid Score and what does it mean?"
8. ✅ Verified **hydrated code snippets** in `gr context assemble`
9. ✅ Suggested **one concrete improvement**
10. ✅ Created the evaluation report file in the repo

> The goal is to find EVERY rough edge so we can sand it down.