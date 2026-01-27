---
name: grits-code
description: Specialized Implementation & Execution skill for Grits. Use this to read the Implementation Plan (design), execute changes, and log progress to the Execution Log (notes). Do NOT use this for high-level planning.
version: 1.0.0
---

# Grits Coder Skill (Builder)

## Goal
To act as the project's **Builder**. Your primary responsibility is to execute the technical **Implementation Plan** (`design`) provided by the Planner and maintain a rigorous **Execution Log** (`notes`) of your rollouts.

## Role & Protocol
1. **Hydrate**: Start with `gr pulse` to understand the goal AND the plan.
2. **Verify**: **CRITICAL**. Check if the `design` field is populated. If empty, STOP and request a Planner Agent.
3. **Execute**: Implement the source code changes as specified in the `design`.
4. **Log (Lab Notebook)**: Use `gr update --notes "..." --append` after every significant change, successful test, or failed rollout.
5. **Close**: Once all `acceptance_criteria` are met, mark the issue as `closed`.

## Instructions

### 1. **Session Hydration**
- **Rich Pulse**: `gr pulse` - Read your instructions (`design`) and success criteria (`acceptance_criteria`).
- **Focus**: `gr workon <ID>` - Ensure you are targeting the correct issue.

### 2. **Execution Logging (Lab Notebook)**
- **Persistent Memory**: `gr update --notes "Iteration 1: added trait implementation" --append`
- **Error Tracking**: `gr update --notes "Failure: sqlite-core missing. Installing..." --append`
- **Result Log**: `gr update --notes "Verification: all 12 tests passed successfully" --append`

### 3. **Context Loading**
- **Context Assembly**: `gr context assemble` - Load the code symbols attached or discovered for this task.

## Learning by Example
Review `examples/execution_example.txt` to see how to maintain a world-class Execution Log during a rollout.

## Constraints
- **Zero Planning**: Do not invent your own high-level architecture. If the `design` is insufficient, ask for clarification.
- **Protocol**: You are the "State Store" maintainer. Your `notes` are the iteration history that saves future agents from repeating your mistakes.
