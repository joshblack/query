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
- [Researcher](../../agents/researcher.agent.md): investigate one bounded
  question, hypothesis, or option and return cited evidence for synthesis.
- [Implementer](../../agents/implementer.agent.md): deliver one ready issue at a
  time in an exclusively leased staging worktree.
- [Reviewer](../../agents/reviewer.agent.md): review the implementation against
  the issue, project decisions, and repository conventions.

Use at most five active sub-agents across all roles. A slot remains occupied
until the agent returns a complete, blocked, failed, or acknowledged
cancellation handoff. Sub-agents must not delegate further.

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

Before fan-out, create a stable artifact root under the primary checkout at
`.tmp/projects/<project-key>/` and assign each agent an exclusive subdirectory.
Persistent plans, research, handoffs, logs, and generated pull request bodies
belong there, not in OS or session temporary directories. GitHub remains the
durable source of truth.

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
8. Create one integration worktree and one staging worktree per active
   implementation lane under the sibling `query.worktrees/` directory.
9. Create the project artifact root and lane directories in the primary
   checkout.
10. Set the Project status, write the first lane-aware checkpoint, and identify
    the next planning, implementation, or integration action.

Use the issue markers, type labels, and Project fields from the shared workflow.
Labels classify work only. The GitHub Project owns lifecycle state.

## Execute incrementally

Work in bounded batches while preserving one canonical stack:

1. Reconstruct the latest state from GitHub and the working tree.
2. Select up to five total planning, research, implementation, or review lanes,
   accounting for every active sub-agent.
3. For implementation, choose only independent `Backlog` tasks with no open
   `blocked by` relationships and complete lane manifests.
4. Set dispatched task issues to `In Progress`, create exclusive staging
   worktrees and branches from the recorded base, and write a checkpoint.
5. Delegate implementation with the full instruction preamble, worktree and
   artifact paths, write lease, base SHA, issue context, decisions, and
   validation requirements.
6. Verify each returned staging head, instruction inventory, local commits,
   validation, write set, and clean worktree.
7. Integrate completed lanes in deterministic dependency order by cherry-picking
   them into canonical stack layers in the integration worktree.
8. Validate the assembled stack, submit it, generate pull request bodies under
   the project artifact root, apply the bodies, and mark the pull requests
   ready.
9. Set integrated tasks to `In Review` and fan out read-only reviews with
   immutable base and head SHAs.
10. Resolve findings from the lowest affected stack layer upward, expire stale
    reviews after SHA changes, and keep all pull request bodies current.
11. Merge the complete approved stack as a unit, then set task and parent
    statuses to `Done`, promote essential state to GitHub, and write the final
    checkpoint.

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

If an existing project issue is supplied, or the user asks to continue prior
work, use the [resume-project skill](../resume-project/SKILL.md).
