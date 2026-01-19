This is a great strategic pivot. You are correct—the "Topological Analysis" (Betti numbers, persistent homology, simplicial complexes) is academic over-engineering for an agentic tool. It adds noise and computational overhead.



However, the \*\*Graph\*\* (Nodes \& Edges) is valuable because that is how you get "Star Neighborhoods" (Context).



Here is your battle plan to strip Grits down to a \*\*Graph-Lite Issue Tracker\*\*.



\### Phase 1: The Purge (Delete "Math" Modules)



You want to keep the \*structure\* (Graph) but delete the \*analysis\* (Math).



\*\*Delete these files entirely:\*\*



1\. `grits-core/src/topology/analysis.rs` (The source of the "flawed logic" / Betti numbers).

2\. `grits-core/src/topology/refactor.rs` (Complex cycle breaking logic).

3\. `grits-core/src/topology/incremental.rs` (Complexity you don't need for a CLI tool).

4\. `grits-cli/src/context.rs` (If it contains complex "inference" logic—keep it simple).



\*\*Modify `grits-core/src/topology/mod.rs`:\*\*



\* \*\*Keep:\*\* `Symbol`, `DependencyEdge`, `SymbolGraph`.

\* \*\*Delete:\*\* References to `analysis`, `refactor`, `incremental`.

\* \*\*Add:\*\* Move the `get\_star` function from `analysis.rs` into `mod.rs` or `graph.rs`. This is a simple Breadth-First Search (BFS) to find connected files. You need this for "Connected Files."



\### Phase 2: The Surgery (Clean the Data Models)



Open `grits-core/src/models.rs`. You need to surgically remove the "Solid" fields that bloat your issues.



\*\*Remove these fields from `struct Issue`:\*\*



```rust

// DELETE THESE:

// pub solid\_volume: Option<String>,

// pub topology\_hash: String,

// pub is\_solid: bool,

// pub feature\_volumes: ...



```



\*\*Keep these:\*\*



```rust

pub affected\_symbols: Vec<String>, // "Connected Files"

pub dependencies: Vec<Dependency>, // Blocking issues



```



\### Phase 3: Simplify the CLI (`grits-cli/src/main.rs`)



Your CLI is currently overloaded. Slash these commands to align with your "Bones" philosophy.



| Command | Action | Rationale |

| --- | --- | --- |

| `Analysis` | \*\*DELETE\*\* | Graph, Duplicates, Volumes, CheckLayers—all noise. |

| `Advisory` | \*\*DELETE\*\* | "Sprint summaries" and "Strategic advice" are bloat. |

| `Refactor` | \*\*DELETE\*\* | Relies on the cycle detection math you just deleted. |

| `ServeMcp` | \*\*DELETE\*\* | You said you are throwing away the MCP server. |

| `Context` | \*\*SIMPLIFY\*\* | Keep `Assemble` (renamed to `Bundle`?), delete `Diff`/`Error`. |



\*\*Refining the "Keepers":\*\*



\#### 1. `gr pulse` (The Dashboard)



Currently, `Pulse` calculates a "Solid Score."



\* \*\*Change:\*\* Rip out the score.

\* \*\*New Logic:\*\* specific output for an agent.

\* "What am I working on?" (Sticky focus).

\* "What is blocked?" (Dependency check).

\* "What changed recently?" (Last 3 git commits).







\#### 2. `gr workon <id>` (The Switch)



Currently, it tries to be too smart (auto-branching, topology checks).



\* \*\*Simplification:\*\*

1\. Sets the "Sticky Focus" (writes ID to `.grits/focus`).

2\. Updates Status to `in-progress`.

3\. \*\*Outputs Context:\*\* Prints the issue description + the list of `affected\_symbols` (files).





\* \*No git branching.\* Let the agent or user decide to branch.

\* \*No topology warnings.\*







\#### 3. `gr star <file>` (The Context Fetcher)



Since you deleted `Analysis` commands, move the logic for "Star Neighborhood" to a top-level command.



\* \*\*Logic:\*\* `SymbolGraph` -> `get\_star(file, depth=1)`.

\* \*\*Agent Usage:\*\* The agent uses this to find "Connected Files" without reading the whole repo.



\### Phase 4: The "Connected Files" Logic (overlap with `ast-grep`)



You mentioned `ast-grep` (sg) overlaps here. You are right.



\* \*\*Current Grits:\*\* Uses a custom TreeSitter parser in Rust to find imports.

\* \*\*Simplified Grits:\*\* You have two options.

1\. \*\*Keep the Rust Parser:\*\* It's already written and fast. It builds the graph for `gr star`.

2\. \*\*Use `ast-grep`:\*\* If you want to delete `grits-core/src/topology/parser.rs`, you can have `gr star` basically run an `sg` scan to find imports.







\*\*Recommendation:\*\* Keep the existing Rust parser (`parser.rs`) but treat it \*only\* as a "Link Detector."



\* It doesn't need to understand code depth.

\* It just needs to know: "File A imports File B."

\* This powers `gr star` instantly without external shell calls.



\### Summary of the "Skeleton" Stack



1\. \*\*`gr issue create/list/show/update`\*\*: Standard CRUD.

2\. \*\*`gr pulse`\*\*: "Where was I?" (Focus + In-Progress).

3\. \*\*`gr workon <id>`\*\*: "I am doing this now." (Sets context).

4\. \*\*`gr star <file>`\*\*: "What is related to this?" (Uses the simplified Graph).



\*\*Agent Skill Integration:\*\*

You create a skill `code-context` that exposes `gr star`.



\* \*\*User:\*\* "Fix the auth bug."

\* \*\*Agent:\*\* `gr show <auth-issue-id>` -> sees `affected\_symbols: \["src/auth.rs"]`.

\* \*\*Agent:\*\* `gr star src/auth.rs` -> returns `\["src/user.rs", "src/session.rs"]`.

\* \*\*Agent:\*\* Now knows exactly which 3 files to edit.



This removes 90% of the code (the math) while keeping 100% of the agentic utility (the context graph).

