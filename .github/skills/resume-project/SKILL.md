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
4. Otherwise select the smallest unblocked `Backlog` task and use the direct
   path by default. Select a batch only when multiple tasks are independent and
   parallel completion is likely to save time.
5. Confirm the selected task is still one reviewable slice. Split or re-plan it
   before coding when it crosses multiple subsystem boundaries or combines the
   first working path with broad hardening.
6. Write a concise checkpoint, then implement directly. Delegate only when the
   task needs specialist context or belongs to a genuinely concurrent batch.
   Create artifact directories only for actual temporary handoffs or generated
   files.
7. Submit a direct task branch as one pull request, or verify coordinated
   staging handoffs and serialize them into the canonical GitHub stack.
8. Apply current pull request bodies and perform a focused acceptance-criteria
   review. Delegate review only when risk or parallelism justifies the
   additional handoff.
9. Resolve concrete findings and revalidate the affected change. For a stack,
   work from the lowest affected layer upward and replace stale reviews.
10. Merge an approved direct task independently, or merge the complete
   coordinated stack as a unit. Update Project status and GitHub state before
   ending the session.
11. Use the `tmp` skill to inspect the exact project artifact directory. Remove
    consumed files and empty directories, and retain only artifacts required by
    an active lane, pending integration, review, or recovery action.

Do not silently change scope. If the next step requires a product or
architectural decision, delegate the investigation and present a recommendation
before editing code.

## Leave another durable checkpoint

End with the Project status and project issue reflecting reality, even when no
code was completed. Record what was learned, every current lane or blocker, and
the exact next action for the next session. Record why any temporary artifact
remains, and clean up everything that is no longer required.
