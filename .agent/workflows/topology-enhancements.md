---
description: Implementation of topology enhancements for advanced simplicial complex x issue tracking
---

# Solid Graph Topology Enhancement - Implementation Plan

## References

- **Paper**: [Solid Graph Philosophy (arxiv.org/html/2512.19736v1)](https://arxiv.org/html/2512.19736v1)
- **Video**: [YouTube Explainer](https://youtu.be/LEmndg6Q6k4?si=E3knh31er6ipidnR)

---

## End Goal Vision

From the paper:

> Applying this "Solid Graph" philosophy to an existing codebase is essentially about moving from **managing files** to **managing logical volumes**.
>
> You can build a Rust-based tool (let's call it a "Topological Linter" or "Structure Guard") that scans your current repo and enforces these principles during AI code generation.

**The Loop:**
```
Parse -> Graph -> Check Topology -> LLM Generate -> Parse -> Graph -> Check Topology -> Save
```

---

## Phase 1: The "CT Scan" (Graph Extraction)

### 0-Simplexes (Nodes)
- Use `tree-sitter` to parse every file
- Every **Function**, **Struct/Class**, and **Interface** becomes a Node
- **CRITICAL**: Do NOT treat files as nodes. Files are arbitrary containers. Logic lives at the symbol level.

### 1-Simplexes (Edges)
- Draw edges for **"Calls"**, **"Imports"**, and **"Inherits"**
- **Weighting**: Assign "strength" to edges. A function call inside a tight loop is a "shorter" (stronger) edge than a one-time config import.

### 2-Simplexes (Volumes)
- Find **Cliques** (3+ nodes all connected)
- If `UserStruct`, `CreateUserFn`, and `UserDBTable` all touch each other = filled triangle (2-simplex)
- Group these into a single "Feature Volume"
- When AI asks to edit `CreateUserFn`, force it to load the entire volume into context

---

## Phase 2: Defining Invariants (Guardrails)

### Invariant 1: Directionality (Acyclic Backbone)
- **Rule**: "The Domain layer cannot import the Infrastructure layer"
- **Implementation**: If an AI suggestion adds an edge that creates a cycle or goes "upstream," reject immediately

### Invariant 2: Connectivity (Betti Numbers)
- **Rule**: "No orphaned islands"
- **Implementation**: If AI deletes a function, check if it increases connected components. If Betti_0 increases, reject.

---

## Phase 3: The Wrapper Workflow

### Context Loading via Star Neighborhoods

Instead of RAG (text similarity), use **Topological Retrieval**:

1. **User Query**: "Fix the bug in the payment retry logic"
2. **System**: Finds the `payment_retry` node
3. **Topological Action**: Grab the **Star** of that node (all connected nodes in the simplicial complex)
4. **Result**: LLM receives the exact "Volume" of code relevant to the feature

### Skeleton-First Edit

1. **Step A (Simulation)**: Apply changes to the Graph in memory (not files yet)
2. **Step B (Homology Check)**: Calculate persistence homology of new graph
   - Did a new hole appear? (circular dependency)
   - Did a volume collapse? (critical logic deleted)
3. **Step C (Commit)**: Only if "Solid" properties preserved, write to files

---

## Implementation Checklist

### Short-term (Required)

#### 1. Directory Scanning
- [ ] Add `gr analysis scan <DIR>` command
- [ ] Recursively find all .rs/.ts/.js files
- [ ] Parse all into single unified `SymbolGraph`
- [ ] Store in memory or temp file for subsequent commands
- [ ] **Files**: `grits-cli/src/main.rs` (add command), possibly new `scanner.rs`

#### 2. Edge Weights
- [ ] Add `weight: f32` field to `DependencyEdge` in `grits-core/src/topology/mod.rs`
- [ ] Parser assigns weight based on context (loop = higher, import = lower)
- [ ] Update analysis to use weights

#### 3. Issue Integration
- [ ] When issue has `affected_symbols`, compute topology
- [ ] Store JSON in `solid_volume` field
- [ ] `gr analysis topology <ISSUE_ID>` returns this stored graph
- [ ] **Files**: `grits-core/src/sync.rs`, `grits-cli/src/main.rs`

### Medium-term (Required)

#### 4. Persistent Graph
- [ ] Save project graph to `.grits/topology.json`
- [ ] Load on startup, update incrementally
- [ ] Add `gr analysis rebuild` to force full rescan
- [ ] **Files**: New `grits-core/src/topology/cache.rs`

#### 5. Diff Comparison
- [ ] Add `gr analysis diff` command
- [ ] Compare current graph to saved graph
- [ ] Report new cycles, collapsed volumes, new orphans
- [ ] **Files**: `grits-cli/src/main.rs`, `grits-core/src/topology/analysis.rs`

#### 6. Context Diff Integration
- [ ] Modify `gr context diff` to include topology warnings
- [ ] If proposed changes would create cycles, warn
- [ ] **Files**: `grits-cli/src/main.rs`

#### 7. Graph Export
- [ ] Add `gr analysis export --format dot|json`
- [ ] DOT format for Graphviz visualization
- [ ] JSON for D3.js or custom UI
- [ ] **Files**: `grits-cli/src/main.rs`

### Long-term (Future)

#### 8. MCP Auto-Validation
- [ ] `validate_architectural_change` already exists
- [ ] Enhance to auto-load star neighborhood
- [ ] Return both validation result AND context
- [ ] **Files**: `grits-cli/src/mcp.rs`

#### 9. VS Code Extension
- [ ] Real-time topology warnings in editor
- [ ] Show feature volume boundaries
- [ ] **Files**: `extension/src/`, new panel

#### 10. Sync Validation
- [ ] Add `--validate-topology` flag to `gr sync`
- [ ] Block sync if topology broken
- [ ] **Files**: `grits-cli/src/main.rs`

#### 11. Betti_2 Computation
- [ ] Find 4-cliques (tetrahedra)
- [ ] Compute 2-voids
- [ ] Expensive - may need optimization
- [ ] **Files**: `grits-core/src/topology/analysis.rs`

---

## Documentation Requirements

For each feature implemented:
- [ ] Update `AGENTS.md` with new commands
- [ ] Update `README.md` if user-facing
- [ ] Add tests in `grits-cli/tests/`
