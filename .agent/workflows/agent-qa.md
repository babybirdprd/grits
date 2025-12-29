---
description: Evaluate the Grits experience from an AI agent perspective
---

# Agent Experience Deep-Dive

This workflow creates a **hands-on evaluation** of Grits. You will complete a real task using Grits, then analyze your experience to identify improvements.

> [!IMPORTANT]
> **Do NOT create a new Git branch for this evaluation.** All work happens on the current branch using Grits' issue tracking only.

---

## Phase 1: Setup (2 min)

### 1.1 Verify Grits Installation
```bash
# Check Grits is working
gr pulse

# If not initialized:
gr onboard
```

### 1.2 Create Your Evaluation Issue
```bash
# Create the tracking issue for this evaluation
gr create "Agent Experience Evaluation - $(date +%Y-%m-%d)" \
  -d "Hands-on QA evaluation following /agent-qa workflow" \
  -t task -p 3

# Lock focus to this issue (no ID needed for subsequent commands)
gr workon <issue-id>
```

---

## Phase 2: The Real Task (15-30 min)

**You must complete an actual task using Grits.** Choose ONE:

### Option A: Bug Hunt
1. Run `gr analysis rebuild` to build the topology cache
2. Run `gr refactor` to find cycles in the codebase
3. Investigate ONE cycle - use `gr analysis star <symbol>` to understand context
4. Document your findings with `gr update --notes "Found cycle: ..."`

### Option B: Code Understanding
1. Pick a file you've never seen: `gr analysis star "path/to/file.rs" --depth 2`
2. Use `gr context assemble --symbol <interesting_symbol>` to extract context
3. Write a summary of what the code does
4. Update issue: `gr update --notes "File X does: ..."`

### Option C: Issue Triage
1. Run `gr list` to see all open issues
2. Run `gr ready` to find actionable work
3. Pick an issue and run `gr inspect <id>` for full context
4. Add symbols: `gr update <id> --add-symbol "path/to/related/file.rs"`

### Option D: Architecture Review
1. Create a `layers.yaml` if none exists (see examples/ in repo)
2. Run `gr analysis check-layers --all`
3. Document any layer violations found
4. Use `gr analysis volumes` to find tightly coupled clusters

**Document everything as you go with `gr update --notes`**

---

## Phase 3: Experience Analysis (10 min)

Now evaluate your experience. Be brutally honest.

### 3.1 Friction Log

Record every point where you got stuck, confused, or had to look something up:

| Moment | What Happened | How You Resolved It | Suggested Fix |
|--------|---------------|---------------------|---------------|
| 1      |               |                     |               |
| 2      |               |                     |               |
| 3      |               |                     |               |

### 3.2 Concept Validation

**The "Latency Test"**: Compare navigation WITH and WITHOUT Grits.

- **Without Grits**: How would you have found the same information? (grep? reading files? asking user?)
- **With Grits**: Which commands saved time? Estimate: __% faster or slower

**The "Aha Test"**: Did Grits reveal something you wouldn't have seen otherwise?

Examples:
- "The cycle detector showed me that ModuleA and ModuleB have a hidden circular dependency"
- "Star neighborhood made me realize this 'utility' function is actually load-bearing"
- "Solid Score dropped after my change, alerting me to architectural damage"

### 3.3 Topology Insights

| Metric | Value | Your Interpretation |
|--------|-------|---------------------|
| Solid Score | __% | |
| Betti₀ (components) | __ | |
| Betti₁ (cycles) | __ | |
| Hottest symbol (PageRank) | __ | |

Did the "simplicial" view (treating code as geometry) help or feel gimmicky?

### 3.4 Session Handoff Test

If you had to hand off to another agent RIGHT NOW:

1. Would they find your issue with `gr pulse`?
2. Would `gr inspect <your-issue>` give them full context?
3. What's missing from the handoff?

---

## Phase 4: Report Generation

### 4.1 Create the Report

Write your evaluation as a markdown file: `agent_experience_qa_v<VERSION>.md`

Structure:
```markdown
# Deep Agent Experience Evaluation: Grits

## 1. Task Completed
[What you actually did]

## 2. Friction Points
[The moments that slowed you down]

## 3. Topological Insights
[What the Betti numbers / Solid Score / PageRank revealed]

## 4. Workflow Ergonomics
[AGENTS.md assessment, CLI usability]

## 5. Session Handoff
[How well does Grits preserve context?]

## 6. Superpower Suggestion
[One feature that would make Grits essential]

## 7. Final Verdict
[Indispensability Score: X/10]
[One-sentence summary]
```

### 4.2 Sync Your Work

```bash
# Update issue with your findings
gr update --description "$(cat agent_experience_qa_v*.md)"

# Close the evaluation issue
gr update --status closed
```

---

## Evaluation Checklist

Before closing, verify:

- [ ] Completed at least ONE real task (Phase 2)
- [ ] Documented friction points as you encountered them
- [ ] Recorded Solid Score and Betti numbers
- [ ] Tested handoff scenario
- [ ] Suggested at least one "superpower" feature
- [ ] Created `agent_experience_qa_v<VERSION>.md` report file
- [ ] Synced issue with findings

---

## Key Questions for Product Improvement

Your evaluation should help answer:

1. **Discoverability**: Could you figure out what to do without reading docs?
2. **Error Messages**: When commands failed, did you understand why?
3. **Documentation**: Is AGENTS.md sufficient? What's missing?
4. **CLI Design**: Any commands that feel "off" (wrong flags, confusing names)?
5. **Topology Value**: Is the "code as geometry" concept actually useful or just novel?

> The goal is to find the friction so we can eliminate it.
