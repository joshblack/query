---
name: resume-project
description: "Resume an issue-backed project from its durable GitHub state. Use when the user asks to continue, pick up, restart, or find the next step for prior project work, especially when they provide a project issue."
---

# Resume a project

Reconstruct project state from GitHub and the repository, then continue from the
smallest safe next action. Do not depend on a previous Copilot session being
available.

Follow the shared [project workflow](../project/references/workflow.md).

## Find the project

If the user supplies an issue, use it. Otherwise:

1. Query the [configured GitHub Project](../project/references/config.md) for
   open items with the `type: project` label.
2. Inspect their `Status`, managed current state, and latest checkpoint.
3. Prefer the most recently updated project whose `Status` is neither `Blocked`
   nor `Done` and that matches the user's description.
4. Present credible candidates if the match is ambiguous.

Do not create a new project because an existing one is temporarily blocked or
stale.

## Reconstruct the state

Read:

- The parent issue body and comments.
- Linked task issues and their open or closed state.
- Native `blocked by` and `blocking` issue relationships.
- GitHub Project membership and `Status` values.
- Linked pull requests and merge state.
- The GitHub stack order, approval state, and rebase status.
- Referenced decision records.
- The recorded integration and staging worktrees, branches, base and head SHAs,
  working trees, and relevant commits.
- The project artifact root and every referenced lane artifact. Missing local
  evidence must be reported, but GitHub state must remain sufficient to resume
  on another machine.

Then summarize:

- The intended outcome and remaining success criteria.
- Completed work and evidence.
- Every active, parked, ready-to-integrate, in-review, blocked, or cancelled
  lane and its `Status`.
- Blockers, unresolved review findings, and decisions needed.
- The next safe planning, implementation, integration, or review action.
- Any mismatch between GitHub and the repository.

## Continue safely

If durable state is consistent, continue with the exact next action from the
latest checkpoint. If it is stale or inconsistent, reconcile it before starting
new work.

For implementation:

1. Reconcile the five-slot ledger from live agents and terminal handoffs.
2. Restore the canonical integration worktree, staging lanes, exclusive branch
   leases, and project artifact directories that are still needed.
3. Prefer integrating a completed lane or resolving a blocked canonical layer
   before dispatching more implementation.
4. Otherwise select an independently implementable batch of unblocked
   `Backlog` tasks, limited by available slots and integration capacity.
5. Write a lane-aware checkpoint before dispatch, then delegate each lane with
   its instruction preamble, worktree, artifact directory, base SHA, write set,
   and full task and project context.
6. Verify committed staging handoffs and serialize them into the canonical
   GitHub stack.
7. Submit the stack, apply current pull request bodies, and fan out read-only
   reviews with immutable base and head SHAs.
8. Resolve findings from the lowest affected layer upward, revalidate, and
   replace stale reviews.
9. Merge the complete stack as a unit, then set the final Project status and
   update GitHub state before ending the session.

Do not silently change scope. If the next step requires a product or
architectural decision, delegate the investigation and present a recommendation
before editing code.

## Leave another durable checkpoint

End with the Project status and project issue reflecting reality, even when no
code was completed. Record what was learned, every current lane or blocker, and
the exact next action for the next session.
