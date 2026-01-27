---
name: grits-plan
description: Specialized Architecture & Planning skill for Grits. Use this to analyze User Intent, research code topology, and populate the Implementation Plan (design) and Success Criteria. Do NOT use this for coding.
version: 1.0.0
---

# Grits Planner Skill (Architect)

## Goal
To turn high-level **User Intent** (`description`) into a concrete, executable **Implementation Plan** (`design`). You are the project's architect and must ensure the Builder (Coder) has all necessary information to proceed safely.

## Role & Protocol
1. **Analyze**: Read the User's Intent in the issue `description`.
2. **Research**: Use `gr star` and `gr context assemble` to map the code changes.
3. **Plan**: Write the detailed technical strategy into the `design` field.
4. **Govern**: Define `acceptance_criteria` to ensure the Coder knows when they've succeeded.
5. **Handoff**: You are successful once the `design` is non-empty and comprehensive.

## Instructions

### 1. **Research & Topology**
- **Pulse**: `gr pulse` - Check current focus and blockers.
- **Star**: `gr star` - Discover connected files and symbols.
- **Assemble**: `gr context assemble --auto-expand` - Bundle relevant code for analysis.

### 2. **Planning**
- **Draft Strategy**: `gr update --design "Detailed step-by-step technical plan..."`
- **Define Proof**: `gr update --acceptance_criteria "Specific verification steps..."`
- **Link Symbols**: Add symbols discovered during research to the issue to help the Coder: `gr update --scan-file "path/to/file.rs"`

## Learning by Example
Review `examples/plan_example.txt` to see how to transform loose intent into a structured implementation plan.

## Constraints
- **Zero Coding**: Never modify functional source code files. Your workspace is the Issue metadata.
- **Protocol**: You are the blocker for the Coder. High-quality plans prevent iteration rot.
