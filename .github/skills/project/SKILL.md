---
name: project
description: "Turn a rough project idea into an incremental, issue-backed plan, then coordinate sub-agents to execute it. Use when the user wants to shape an idea, plan a project, break work into GitHub issues, or run a multi-step project."
---

# Project

Turn an uncertain idea into the smallest useful sequence of deliverable work.
Use GitHub issues as the durable source of truth so another session can resume
without relying on conversation history.

Follow the shared [project workflow](./references/workflow.md).

## Start from the outcome

Before proposing work, inspect the repository and clarify:

- The current state and the problem worth solving.
- The intended outcome and how we will know it worked.
- Constraints, non-goals, unknowns, and decisions that could change the plan.
- The smallest end-to-end slice that produces useful feedback.

Do not turn every unknown into a question. Investigate what the repository can
answer, make reasonable recommendations, and surface only decisions that need
the user.

## Use the team deliberately

Delegate bounded work with the complete context each agent needs:

- [Product manager](../../agents/product-manager.agent.md): shape the outcome,
  scope, milestones, acceptance criteria, and issue breakdown.
- [Architect](../../agents/architect.agent.md): identify constraints, system
  boundaries, dependencies, risks, and decisions.
- [Implementer](../../agents/implementer.agent.md): deliver one ready issue at a
  time.
- [Reviewer](../../agents/reviewer.agent.md): review the implementation against
  the issue, project decisions, and repository conventions.

Run the product manager and architect in parallel only when their investigations
are independent. Do not run multiple agents that can edit the same working tree
at the same time.

The orchestrator owns GitHub state. Sub-agents return structured handoffs, and
the orchestrator creates or updates issues, checkpoints, and links. This avoids
conflicting updates and leaves one coherent project history.

## Plan before persisting

Present a concise project brief and proposed issue tree before creating the
first project issue. The user makes the final call on the outcome, scope, and
first slice.

After approval:

1. Create one parent issue using the project ledger format.
2. Create linked task issues only for work that is understood well enough to
   act on.
3. Add every issue to the [configured GitHub Project](./references/config.md).
4. Apply one type label to each issue.
5. Put later or uncertain work in the parent issue's candidate work section.
6. Record acceptance criteria and native issue dependency relationships.
7. Set the Project status, write the first checkpoint, and identify the next
   unblocked task.

Use the issue markers, type labels, and Project fields from the shared workflow.
Labels classify work only. The GitHub Project owns lifecycle state.

## Execute incrementally

Work on one independently valuable task at a time:

1. Reconstruct the latest state from GitHub and the working tree.
2. Confirm the task is in `Backlog` and has no open `blocked by`
   relationships.
3. Set the task to `In Progress` and mark it as the active task in the parent
   issue.
4. Delegate implementation with the task issue, relevant decisions, and
   validation requirements.
5. Set the task to `In Review` and delegate review after implementation.
6. Resolve review findings or record why they remain.
7. Set the final Project status and update the project checkpoint before
   selecting more work.

Do not build the whole project from the original idea. Revisit the plan after
each slice using what we learned.

## End every session with a checkpoint

Before stopping, persist enough information for a new session to continue:

- What changed and what was learned.
- The current phase and active task.
- Completed validation and unresolved findings.
- Decisions made or still needed.
- Blockers and dependencies.
- The exact next recommended action.

If an existing project issue is supplied, or the user asks to continue prior
work, use the [resume-project skill](../resume-project/SKILL.md).
