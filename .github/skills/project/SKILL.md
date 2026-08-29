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

Default to direct orchestration for one bounded, ready task. The orchestrator
may inspect, implement, validate, commit, and submit that task in its dedicated
worktree without delegating each phase. Use a sub-agent only when separate
context, specialist judgment, or genuine parallelism is worth the handoff cost.

Delegate bounded work with the complete context each agent needs:

- [Product manager](../../agents/product-manager.agent.md): shape the outcome,
  scope, milestones, acceptance criteria, and issue breakdown.
- [Architect](../../agents/architect.agent.md): identify constraints, system
  boundaries, dependencies, risks, and decisions.
- [Researcher](../../agents/researcher.agent.md): investigate one bounded
  question, hypothesis, or option and return cited evidence for synthesis.
- [Implementer](../../agents/implementer.agent.md): deliver one ready issue at a
  time in an exclusively leased staging worktree.
- [Reviewer](../../agents/reviewer.agent.md): review the implementation against
  the issue, project decisions, and repository conventions.

Use at most five active sub-agents across all roles. A slot remains occupied
until the agent returns a complete, blocked, failed, or acknowledged
cancellation handoff. Sub-agents must not delegate further.

Do not fan out a single sequential task. Fan out implementation only when at
least two ready tasks are independent, the integration boundary is already
clear, and parallel completion is likely to save time. A bounded specialist
investigation may still use one sub-agent when it avoids loading substantial
unrelated context into the orchestrator.

For planning or research fan-out, define one shared question, evidence standard,
evaluation rubric, synthesis owner, and mutually exclusive lane assignments.
Use researchers for bounded evidence gathering and an architect on
`gpt-5.6-sol` for final technical synthesis. Do not dispatch multiple agents
with the same generic question.

For implementation fan-out, select an antichain of actionable task issues and
give every implementer an exclusive staging branch, worktree, write set, base
SHA, and artifact directory. Never run multiple editing agents in the same
worktree or on the canonical stack. Serialize completed staging commits through
the integration worktree.

The orchestrator owns GitHub state. Sub-agents return structured handoffs, and
the orchestrator creates or updates issues, checkpoints, and links. This avoids
conflicting updates and leaves one coherent project history.

Every delegation must require the agent to read the root and applicable nested
`AGENTS.md` files, its agent contract, and every relevant skill instruction
before acting. Agents may use all available repository-local and global skills,
but skills do not expand tool access, scope, write leases, or GitHub ownership.
Reject handoffs that omit instruction paths, derived obligations, environment
setup, or instruction-derived validation.

Before fan-out, invoke the `tmp` skill, create an artifact root under the primary
checkout at `.agents/tmp/projects/<project-key>/`, and assign each agent an
exclusive subdirectory. For the direct path, create temporary artifacts only
when a concrete plan, handoff, log, or generated body needs to persist. Do not
create empty coordination scaffolding. GitHub remains the durable source of
truth. Clean up each artifact after it is consumed and its essential state is
promoted.

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
7. Plan the issue dependency graph, independently implementable batches, and
   deterministic canonical stack integration order.
8. Choose the direct path for one bounded ready task, or the coordinated path
   for concurrent lanes or a multi-layer stack.
9. Create only the worktrees and artifact directories required by the chosen
   path under the sibling `query.worktrees/` directory and the primary checkout.
10. Set the Project status, write the first lane-aware checkpoint, and identify
    the next planning, implementation, or integration action.

Use the issue markers, type labels, and Project fields from the shared workflow.
Labels classify work only. The GitHub Project owns lifecycle state.

## Execute incrementally

Work in bounded slices. Use a canonical stack only for the coordinated path:

1. Reconstruct the latest state from GitHub and the working tree.
2. Choose the direct path unless concurrent or specialist work has a clear
   expected benefit.
3. For implementation, choose only independent `Backlog` tasks with no open
   `blocked by` relationships and complete lane manifests.
4. Confirm that each selected task is a small reviewable slice. Split it before
   implementation when it crosses multiple subsystem boundaries, requires more
   than one independent behavioral proof, or combines a walking skeleton with
   production hardening.
5. Set active task issues to `In Progress`, create the required worktree and
   branch from the recorded base, and write a concise checkpoint.
6. Implement directly, or delegate only when the coordinated path applies. In
   either case, produce a committed checkpoint after focused validation before
   beginning broader review or integration.
7. On the direct path, submit the task branch as one pull request. On the
   coordinated path, integrate completed lanes in deterministic dependency order
   and submit the canonical stack.
8. Run the instruction-required validation, fill the pull request template, and
   mark the pull request ready.
9. Set submitted tasks to `In Review` and perform a focused review against the
   acceptance criteria. Delegate a read-only review only for elevated risk,
   substantial changes, or useful parallel review capacity.
10. Resolve concrete findings, revalidate the affected change, and keep pull
    request bodies current. For a stack, work from the lowest affected layer
    upward and expire reviews made stale by SHA changes.
11. Merge an approved direct task independently, or merge a complete coordinated
    stack as a unit. Update task and parent status based on the project outcome,
    promote essential state to GitHub, and write the final checkpoint.
12. Inspect the exact project artifact directory and remove temporary files and
    empty lane directories that no active lane, pending integration, review, or
    recovery action still needs.

Do not build the whole project from the original idea. Revisit the plan after
each slice using what we learned.

## End every session with a checkpoint

Before stopping, persist enough information for a new session to continue:

- What changed and what was learned.
- The current phase and every active, parked, or blocked lane.
- The integration worktree, staging worktrees, artifact root, branch leases,
  base and head SHAs, canonical stack, and pull requests.
- Completed validation and unresolved findings.
- Decisions made or still needed.
- Blockers and dependencies.
- The exact next planning, lane, integration, or review action.
- Temporary artifacts retained for active work and the reason each one remains.

Clean up consumed temporary artifacts before ending the session. Do not retain
them only because they might be useful later.

If an existing project issue is supplied, or the user asks to continue prior
work, use the [resume-project skill](../resume-project/SKILL.md).
