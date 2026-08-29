# Project workflow

GitHub issues are the durable project ledger. Conversation history, local notes,
and agent state are supporting context, not the source of truth.

Issue labels classify the kind of work. The
[configured GitHub Project](./config.md) owns lifecycle state through its
built-in `Status` field. Do not encode lifecycle state in labels.

## Ownership

The orchestrator is the only participant that creates or updates issues,
checkpoints, links, type labels, and GitHub Project fields. Sub-agents inspect,
implement, review, and return structured handoffs.

This ownership rule prevents concurrent agents from overwriting each other and
makes the project understandable from GitHub alone.

## Parent project issue

Apply the `type:project` label and include this marker:

```markdown
<!-- copilot-project -->
```

Use this body shape:

```markdown
<!-- copilot-project -->

The current state and the outcome this project tracks.

## Outcome

The user-visible result and why it matters.

## Success criteria

- [ ] An observable result.

## Scope

### Requirements

- A requirement.

### Non-goals

- A boundary.

### Constraints

- A constraint.

## Decisions

- A decision or a link to a decision record.

## Work

- [ ] #123

## Candidate work

- Work that is not understood or committed enough for a task issue.

<!-- project-state:start -->
## Current state

- **Active task:** #123 or none
- **Last checkpoint:** YYYY-MM-DD
- **Next action:** A concrete action another session can take.
- **Blockers:** None, or linked blockers and the decision needed.
<!-- project-state:end -->
```

The orchestrator may update the managed current state block in place. Preserve
all content outside its markers.

## Task issues

Apply exactly one of these type labels:

- `type:feature` for a user-visible capability or meaningful enhancement.
- `type:bug` for behavior that does not work as intended.
- `type:task` for enabling, maintenance, investigation, or implementation work
  that is not independently a feature or bug.

Use this marker, replacing the parent number:

```markdown
<!-- copilot-project-task parent=#123 -->
```

Each task must be independently reviewable and should produce useful progress.
Use this body shape:

```markdown
<!-- copilot-project-task parent=#123 -->

This issue tracks the result this slice delivers.

## Context

Why this slice exists and what project decision or constraint applies.

## Acceptance criteria

- [ ] An observable criterion.

## Validation

- The checks that demonstrate the criteria are met.

## Out of scope

- Related work that does not belong in this slice.
```

Link task issues from the parent issue with a task list. Use native sub-issues
when available, but do not make the workflow depend on them.

Record task ordering with native `blocked by` and `blocking` issue
relationships. Do not duplicate dependency lists in issue bodies. A task is
unblocked only when it has no open `blocked by` relationships.

## GitHub Project

Add every parent and task issue to the repository's configured GitHub Project.
Use the built-in `Status` field as the lifecycle source of truth:

- `Backlog`: Defined work that is not actively being implemented.
- `In Progress`: The task currently being implemented or reviewed.
- `In Review`: Implementation is complete and review is active.
- `Blocked`: The issue has one or more open native `blocked by` relationships,
  or the checkpoint identifies a decision that prevents progress.
- `Done`: Acceptance criteria and validation are complete.

New items start in `Backlog`. Select a task only when its acceptance criteria
are actionable and it has no open `blocked by` relationships. When a blocker is
added, move the issue to `Blocked`. When all blockers close, move it back to
`Backlog`.

The parent project issue remains `In Progress` while committed child work is
active. Set it to `Blocked` only when no project work can proceed. Set it to
`Done` only when the project success criteria are complete or the issue records
why the project ended.

Prefer project views grouped or filtered by `Status` and type label. Do not add
lifecycle labels.

## Checkpoints

Add a checkpoint comment after planning, after each completed or blocked task,
and before ending a session:

```markdown
## Checkpoint

- **Status:** In Progress
- **Completed:** What changed since the previous checkpoint.
- **Learned:** Information that affects the plan.
- **Decisions:** Decisions made or still needed.
- **Validation:** Checks completed and unresolved findings.
- **Active task:** #123
- **Blockers:** None, or the blocker and owner.
- **Next action:** The exact next recommended action.
```

Update the parent issue's managed current state block to match the latest
checkpoint.

## Discovery

When no issue number is supplied:

1. Query the configured GitHub Project for open items with the `type:project`
   label.
2. Prefer an issue explicitly named by the user.
3. Otherwise prefer the most recently updated project whose `Status` is neither
   `Blocked` nor `Done`.
4. If more than one issue is a credible match, present the candidates rather
   than guessing.

## Reconciliation

Before selecting work, compare:

- The parent issue and latest checkpoint.
- Open and closed linked task issues.
- GitHub Project membership and `Status` values.
- Linked pull requests and their merge state.
- Applicable decision records.
- The current branch, working tree, and relevant commits.

Issues record intent and durable context. The GitHub Project records lifecycle.
The repository records what has actually been implemented. When they disagree,
report the mismatch and update the Project and checkpoint after resolving it.

## Execution rules

- Select only a `Backlog` task with no open `blocked by` relationships.
- Move active tasks to `In Progress` before implementation.
- Move implemented tasks to `In Review` before review.
- Move issues to `Blocked` when an open native dependency prevents progress.
- Move unblocked issues back to `Backlog`.
- Run one editing agent at a time in a shared working tree.
- Review before marking a task complete.
- Do not close a task only because code exists. Confirm its acceptance criteria
  and validation.
- Re-plan when implementation reveals a false assumption or changes the value
  of later work.
- Keep issue bodies current and use comments as the chronological activity log.

## Required repository setup

Create these type labels if they do not exist:

- `type:project`
- `type:feature`
- `type:bug`
- `type:task`

Configure one GitHub Project and record its owner, number, and URL in
[config.md](./config.md). If project access is unavailable, stop and report the
missing authentication scope instead of falling back to lifecycle labels.

Native issue dependencies are available in personal and organization-owned
repositories. Prefer them over dependency text or dependency labels.
