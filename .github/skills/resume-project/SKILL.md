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
   open items with the `type:project` label.
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
- Referenced decision records.
- The current branch, working tree, and relevant commits.

Then summarize:

- The intended outcome and remaining success criteria.
- Completed work and evidence.
- The active task and its `Status`, if it is still valid.
- Blockers, unresolved review findings, and decisions needed.
- The next `Backlog` task with no open `blocked by` relationships.
- Any mismatch between GitHub and the repository.

## Continue safely

If durable state is consistent, continue with the exact next action from the
latest checkpoint. If it is stale or inconsistent, reconcile it before starting
new work.

For implementation:

1. Select one unblocked `Backlog` task.
2. Set it to `In Progress`, update the parent issue's active task, and write a
   checkpoint.
3. Delegate to the implementer with the full task and project context.
4. Set it to `In Review` and delegate review.
5. Resolve findings.
6. Set the final Project status and update GitHub state before ending the
   session.

Do not silently change scope. If the next step requires a product or
architectural decision, delegate the investigation and present a recommendation
before editing code.

## Leave another durable checkpoint

End with the Project status and project issue reflecting reality, even when no
code was completed. Record what was learned, the current blocker or active task,
and the exact next action for the next session.
