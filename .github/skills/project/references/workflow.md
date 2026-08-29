# Project workflow

GitHub issues are the durable project ledger. Conversation history, local notes,
and agent state are supporting context, not the source of truth.

Issue labels classify the kind of work. The
[configured GitHub Project](./config.md) owns lifecycle state through its
built-in `Status` field. Do not encode lifecycle state in labels.

Each project is implemented as one GitHub stack in one dedicated git worktree.
The primary repository checkout remains available for user work. Each committed
task becomes one branch and pull request in the stack, ordered from foundational
work at the bottom to dependent work at the top. Land the project with the stack
merge command rather than merging its pull requests individually.

## Ownership

The orchestrator is the only participant that creates or updates issues,
checkpoints, links, type labels, and GitHub Project fields. Sub-agents inspect,
implement, review, and return structured handoffs.

This ownership rule prevents concurrent agents from overwriting each other and
makes the project understandable from GitHub alone.

## Parent project issue

Apply the `type: project` label and include this marker:

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
- **Worktree:** An absolute path, or not created
- **Stack:** The stack number and branch chain, or not submitted
- **Pull request:** The active task pull request, or none
- **Last checkpoint:** YYYY-MM-DD
- **Next action:** A concrete action another session can take.
- **Blockers:** None, or linked blockers and the decision needed.
<!-- project-state:end -->
```

The orchestrator may update the managed current state block in place. Preserve
all content outside its markers.

## Task issues

Apply exactly one of these type labels:

- `type: feature` for a user-visible capability or meaningful enhancement.
- `type: bug` for behavior that does not work as intended.
- `type: task` for enabling, maintenance, investigation, or implementation work
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
- `In Progress`: The task currently being implemented.
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

A task remains `In Review` after its pull request is approved. Move all tasks in
the stack to `Done` only after the full stack lands and the task acceptance
criteria are confirmed against the default branch.

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
- **Worktree:** The project worktree path.
- **Stack:** The stack number and branch chain.
- **Pull request:** The active task pull request.
- **Blockers:** None, or the blocker and owner.
- **Next action:** The exact next recommended action.
```

Update the parent issue's managed current state block to match the latest
checkpoint.

## Discovery

When no issue number is supplied:

1. Query the configured GitHub Project for open items with the `type: project`
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
- The GitHub stack order and approval state.
- Applicable decision records.
- The project worktree, current branch, working tree, and relevant commits.

Issues record intent and durable context. The GitHub Project records lifecycle.
The repository records what has actually been implemented. When they disagree,
report the mismatch and update the Project and checkpoint after resolving it.

## Worktree and stack setup

Create the project worktree after the project brief and issue tree are approved,
before implementation starts:

1. Fetch the default branch and confirm the primary checkout does not contain
   project changes that need to move.
2. Create a sibling worktree outside the primary checkout. Use a stable,
   descriptive path based on the repository and parent issue, such as
   `../query-worktrees/project-123`. Create the first branch and worktree with
   `git worktree add -b <branch> <path> <default-branch>`.
3. Create the bottom stack branch in that worktree from the current default
   branch. Use a descriptive branch name that includes the parent or task issue
   number.
4. Run `gh stack init <branch>` from the project worktree. Always pass branch
   names, and configure `rerere.enabled` and `remote.pushDefault` first so the
   command remains non-interactive.
5. Record the worktree path and stack branch chain in the parent issue's managed
   current state and checkpoint.

Reuse the same project worktree for every layer. Add each later branch with
`gh stack add <branch>` only when its task becomes active. Do not create branches
for candidate or insufficiently understood work. Keep unrelated changes out of
the project worktree.

Plan stack layers in dependency order. A lower branch must contain everything a
higher branch needs. If implementation reveals that a lower layer must change,
navigate to that branch, commit the change there, run
`gh stack rebase --upstack`, and return to the active branch.

## Pull request and review flow

Each task branch has one pull request. A task is ready for review only after its
changes are committed, validated, submitted, and described:

1. Run `gh stack submit --auto` from the project worktree to push the stack and
   create or update draft pull requests.
2. Fill out [the pull request template](../../../pull_request_template.md) for
   the active task. Include the parent project issue, close the task issue with
   a supported closing keyword, and describe the stack position, behavior
   changed, validation, and focused review notes.
3. Update the pull request with `gh pr edit --body-file <path>`. The stack
   extension generates titles and initial bodies, so the explicit edit is
   required.
4. Mark the active pull request ready with `gh pr ready <number>`.
5. Move the task to `In Review`, link the pull request in the checkpoint, and
   delegate the reviewer against that pull request.

Creating the pull request before delegating review lets the user and reviewer
inspect the same change at the same time. The reviewer is read-only and must not
modify the project worktree or pull request. Address findings on the same stack
branch, rebase branches above it when needed, submit the stack again, and keep
the pull request body current.

An approved pull request remains open while later stack layers are implemented.
After every committed task is approved and the project success criteria are
satisfied, synchronize the stack and use GitHub's atomic stack merge with
`gh stack merge <stack-number> --yes` and an explicit repository-appropriate
`--merge`, `--squash`, or `--rebase` flag. Do not use `gh pr merge` for project
stack pull requests. If any pull request cannot merge, none of the stack lands.

After the stack lands, run `gh stack sync --prune`, confirm the default branch
contains the project outcome, move the task issues and parent issue to `Done`,
write the final checkpoint, and remove the project worktree only when it is
clean.

## Execution rules

- Select only a `Backlog` task with no open `blocked by` relationships.
- Move active tasks to `In Progress` before implementation.
- Create or switch to the task's stack branch in the project worktree before
  implementation.
- Submit a pull request and fill its body before moving the task to `In Review`.
- Move issues to `Blocked` when an open native dependency prevents progress.
- Move unblocked issues back to `Backlog`.
- Run one editing agent at a time in the project worktree.
- Review before marking a task complete.
- Do not merge task pull requests individually.
- Do not close a task only because code exists or review is complete. Confirm
  its acceptance criteria and validation after the full stack lands.
- Re-plan when implementation reveals a false assumption or changes the value
  of later work.
- Keep issue and pull request bodies current, and use comments as the
  chronological activity log.

## Required repository setup

Create these type labels if they do not exist:

- `type: project`
- `type: feature`
- `type: bug`
- `type: task`

Configure one GitHub Project and record its owner, number, and URL in
[config.md](./config.md). If project access is unavailable, stop and report the
missing authentication scope instead of falling back to lifecycle labels.

Native issue dependencies are available in personal and organization-owned
repositories. Prefer them over dependency text or dependency labels.

Install and authenticate the `github/gh-stack` GitHub CLI extension. The
repository must have GitHub stacked pull requests enabled. If `gh stack submit`
exits because stacks are unavailable, stop and report the missing repository
capability instead of creating unrelated pull requests.
