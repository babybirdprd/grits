---
description: Evaluate the Grits experience from an AI agent perspective
---

# Agent Experience Q&A Workflow

Use this workflow to conduct a structured evaluation of Grits from an AI agent perspective. This captures targeted feedback that directly informs product improvement.

## Prerequisites
- Grits must be initialized in the project (`gr onboard`)
- You should have used Grits for at least one task before running this evaluation

## Steps

1. **Create evaluation issue**
   ```bash
   gr create --title "Agent Experience Evaluation - [DATE]" --labels "meta,feedback"
   ```

2. **Complete the Q&A Framework**
   
   Work through each section of the framework below, answering honestly based on your experience.

### Section 1: First Impressions
- Did you find `AGENTS.md` or equivalent documentation? (Y/N)
- Time to first useful action: ___ minutes
- Friction points encountered: ___

### Section 2: Core Functionality (Rate 1-5)
| Task | Rating | Friction |
|------|--------|----------|
| Create issue | | |
| Update status | | |
| Find existing issues | | |
| Link related issues | | |

Did Grits reduce files you had to open? (>50%, 20-50%, No difference, Added overhead)

### Section 3: VS Code Extension
Rate sidebar usefulness: (1-5)
Rate dashboard usefulness: (1-5)

Views assessment:
- List: Useful? Y/N
- Kanban: Useful? Y/N  
- Graph: Useful? Y/N
- 3D Topology: Useful? Y/N
- Focus: Useful? Y/N

### Section 4: Agent-Specific Workflows
- Session handoff: Can you capture state for next agent? Y/N
- Error-to-issue: Does `gr suggest` help? Y/N
- Architectural analysis: Would you use in normal work? Y/N

### Section 5: The Indispensability Test
**Critical question**: Do you feel you CANNOT do agent-driven development without Grits?
- [ ] Yes, essential
- [ ] Partially useful
- [ ] Could use alternatives

Recommendation score (0-10): ___

### Section 6: Improvement Suggestions
- Top 3 pain points: ___
- Top 3 feature requests: ___
- What should be removed? ___

3. **Record findings**
   ```bash
   gr update <issue-id> --description "## Evaluation Results\n[Paste your completed assessment]"
   ```

4. **Sync to repo**
   ```bash
   gr sync
   ```
